---
staticPath: fuzzillipart2
Title: "Fuzzilli Deep Dive: Part 2"
Description: Fuzzilli Coverage Internals
Date: 2024-12-21
tags:
  - Fuzzing
  - Fuzzilli
  - Rust
---

# Getting  V8 code coverage

Below is how I implemented Fuzzillis' coverage mechanism in Rust. 

```rust
const SHM_SIZE: usize = 0x200000;
    let res = shm_open("test", OFlag::O_CREAT | OFlag::O_RDWR, Mode::S_IRWXU).unwrap();
    ftruncate(res, SHM_SIZE.try_into().unwrap()).unwrap();
    let non_zero = NonZeroUsize::new(SHM_SIZE).unwrap();
    let shared = unsafe {
        mmap(
            None,
            non_zero,
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_SHARED,
            res,
            0,
        )
        .unwrap()
    };
```

First I create a shared memory called test. I then call ftruncate to set its size. I then map it into my processes using `mmap`. I then spawn d8 built with Fuzzilli enabled. Built with the following commands:

```
gn gen out/fuzzilli --args='is_debug=false dcheck_always_on=true v8_static_library=true v8_enable_verify_heap=true v8_fuzzilli=true sanitizer_coverage_flags="trace-pc-guard" target_cpu="x64"'
ninja -C ./out/fuzzilli/ d8
```

Finally, I spawn a d8 process while exporting the same name of the shared memory in this case `test`. 

```
export SHM_ID=test && ./d8
```

You should see something like this.

![[Pasted image 20241216130951.png]]

On the rust side, I have it waiting for user input. So that it reads coverage only after d8 is started. 

```rust
let mut guess = String::new();
    io::stdin().read_line(&mut guess).expect("Failed to read line");
  
    let shmem: *mut ShmemData = shared as *mut ShmemData;

    unsafe {
        if !shmem.is_null() {
            println!("No edges {:#?}", (*shmem).num_edges); 
            println!("Raw cov: ");
            let mut j = 0;
            for i in ((*shmem).edges) {
                print!("{}",i );

                if j > 40 {
                    break
                }

                j+=1;
            }
            println!("");
        } else {
            println!("Shmem is null!");
        }
    }
```

Once d8 is initialized, I just have to run some JavaScript and then get coverage. The above code prints the total number of edges and the first 40 bytes of the coverage bitmap.

![[Pasted image 20241216131234.png]]


# On the v8 side

Coverage is implemented using the `v8/src/fuzzilli/cov.cc` code. First, it opens a shared memory region. It then sets the number of edges to 

```
shmem->num_edges = static_cast<uint32_t>(stop - start);
```

Where stop - start are the parameters provided to
```
__sanitizer_cov_trace_pc_guard_init
```

This is a special function that is called by instrumented code compiled with LLVM. You can learn more about this feature [here](https://clang.llvm.org/docs/SanitizerCoverage.html). The instrumentation also calls `__sanitizer_cov_trace_pc_guard` which keeps track of which edges have reached by updating the shared memory. 

```
shmem->edges[index / 8] |= 1 << (index % 8);
```

The above code sets the bit in `shmem->edges` to true corresponding with which edge is visited. The entire source of the code is 

```c
extern "C" void __sanitizer_cov_trace_pc_guard(uint32_t* guard) {
  // There's a small race condition here: if this function executes in two
  // threads for the same edge at the same time, the first thread might disable
  // the edge (by setting the guard to zero) before the second thread fetches
  // the guard value (and thus the index). However, our instrumentation ignores
  // the first edge (see libcoverage.c) and so the race is unproblematic.
  uint32_t index = *guard;
  shmem->edges[index / 8] |= 1 << (index % 8);
  *guard = 0;
}
```

Guard values are values the compiler inserts into every basic block. A basic block is a piece of code that doesn't change the control flow. The following is an example of how a guard value might be used.

```c
uint32_t guard = UNIQUE_ID; 
__sanitizer_cov_trace_pc_guard(&guard); i
f (x > 0) { 
	doSomething(); 
}
```

# Talking to V8 from the fuzzing engine

First I spawn a d8 a process. D8 is the v8 shell.

```rust
let mut fuzzilli = Command::new("/home/ubuntu/v8/out/fuzzilli/d8")
	.env("SHM_ID", "test")
	.stdin(std::process::Stdio::piped()) // Enable writing to stdin
	.stdout(std::process::Stdio::piped()) // Enable reading from stdout
	.spawn()
	.expect("couldn't spawn d8 shell");
```

Then I take both stdin and stdout and create a buffered reader to read lines from stdout.

```rust
let mut child_stdin = fuzzilli.stdin.take().expect("Failed to open stdin");
let stdout = fuzzilli.stdout.take().expect("Failed to open stdout");
let mut reader = BufReader::new(stdout).lines();
```

I'm now able to read and write to d8!

```rust
// Write to stdin and read from stdout multiple times
for i in 1..=3 {
	let input = "var x = 1;\n".to_string();
	child_stdin.write_all(input.as_bytes()).await.unwrap();
	println!("Sent: {}", input.trim());

	if let Some(output) = reader.next_line().await.unwrap() {
		println!("Received: {}", output);
	}
}
```
#### It lives!
![[Pasted image 20241221150500.png]]

# Implementing evaluate_edges 

Now that I'm able to execute arbitrary javascript. Let's implement a coverage function that detects whether a javascript execution reached new edges or not. I'll be implementing the algorithms found in `fuzzilli/Sources/libcoverage/coverage.c` First I define the coverage struct:
```rust
pub struct CovContext {
    id: u8,
    should_track_edges: bool,
    virgin_bits: Box<[u8]>,
    crash_bits: Box<[u8]>,
    num_edges: u32,
    bitmap_size: u32,
    found_edges: u32,
    shmem_data: *mut ShmemData,
    edge_count: Option<Box<[u32]>>,
}
```


To keep things simple I did not implement edge tracking. After d8 starts and receives output from the child process. I initialize the coverage struct:

```rust
pub fn cov_finish_evaluation(shmem: *mut ShmemData) -> CovContext {
    let mut num_edges = unsafe { (*shmem).num_edges };
    if num_edges == 0 {
        panic!("Couldn't get the number of edges");
    }

    num_edges += 1;

    if num_edges > MAX_EDGES.try_into().unwrap() {
        panic!("[LibCoverage] Too many edges\n");
    }

    let mut bitmap_size = (num_edges + 7) / 8;
    bitmap_size += 7 - ((bitmap_size - 1) % 8);

    let mut virgin_bits = vec![0xff; bitmap_size.try_into().unwrap()];
    let mut crash_bits = vec![0xff; bitmap_size.try_into().unwrap()];

    clear_edge(&mut virgin_bits, 0);
    clear_edge(&mut crash_bits, 0);

    CovContext {
        id: 0,
        should_track_edges: false,
        virgin_bits: virgin_bits.into_boxed_slice(),
        crash_bits: crash_bits.into_boxed_slice(),
        num_edges: num_edges,
        bitmap_size: bitmap_size,
        found_edges: 0,
        shmem_data: shmem,
        edge_count: None,
    }
}
```

 I implemented the `internal_evaluate` which checks if the last execution gained new coverage.

```rust
fn internal_evaluate(
    context: &mut CovContext,
    virgin_bits: Box<[u8]>,
    new_edges: &mut NewEdges,
) -> u64 {
    // SAFETY: Assuming the shmem_data is valid and properly aligned
    let edges = unsafe {
        std::slice::from_raw_parts(
            context.shmem_data as *const u64,
            (context.bitmap_size as usize) / 8,
        )
    };

    new_edges.count = 0;
    new_edges.edge_indices.clear();

    // Collect edges to be cleared in a temporary buffer to avoid borrowing conflicts
    let mut indices_to_clear = Vec::new();

    // First Pass: Immutable Read
    for (offset, (&current_chunk, virgin_chunk)) in edges
        .iter()
        .zip(virgin_bits.chunks_exact(8))
        .enumerate()
    {
        if current_chunk != 0
            && (current_chunk & u64::from_ne_bytes(virgin_chunk.try_into().unwrap())) != 0
        {
            let index = (offset as u32) * 64;

            for i in index..index + 64 {
                if edge(context.virgin_bits.as_ref(), i.try_into().unwrap()) == 1
                    && edge(virgin_bits.as_ref(), i.try_into().unwrap()) == 1
                {
                    indices_to_clear.push(i);
                }
            }
        }
    }

    // Second Pass: Mutable Write
    for &i in &indices_to_clear {
        clear_edge(context.virgin_bits.as_mut(), i.try_into().unwrap());
        new_edges.count += 1;
        new_edges.edge_indices.push(i as u64);
    }

    new_edges.count
}
```

It works by iterating over the coverage map and comparing the previous bitmap and the current map. If there is a difference we've found a new edge. Finally, I implemented `cov_evaluate` which is a wrapper around `internal_evaluate`.

```rust
pub fn cov_evaluate(context: &mut CovContext) -> bool {
    let mut new_edges = NewEdges {
        count: 0,
        edge_indices: Vec::new(),
    };

    let num_new_edges: u32 = internal_evaluate(context, context.virgin_bits.clone(), &mut new_edges).try_into().unwrap();
    // TODO found_edges should also include crash bits
    context.found_edges += num_new_edges;
    return num_new_edges > 0;
}
```

Here's it all in action!

![[Pasted image 20241222172845.png]]

First, I set the variable x to one which increases coverage. Then I just add one to it which doesn't change coverage.