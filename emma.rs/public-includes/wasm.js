function sleep(time) {
    return new Promise((resolve) => setTimeout(resolve, time));
  }
  
  // Initialize the WASM module dynamically
  (async () => {
    try {
      const { default: init, select_random_string } = await import('/wasm_random_string.js'); // Adjust path if needed
  
      await init(); // Initialize WASM
      await sleep(500); // Wait 500ms for initialization
  
      const randomString = select_random_string();
      document.querySelector('.highlight').innerText = randomString;
    } catch (error) {
      console.error('Failed to load WASM module:', error);
    }
  })();