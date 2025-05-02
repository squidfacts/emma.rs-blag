use fxhash::hash32;
use std::fs;
use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use std::str;
use dotenvy;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

fn process_blog(
    hugo_base_dir: &String,
    obsidan_img_folder: &String,
    filename: &str,
    _obsidan_dir: &String,
) {
    fs::create_dir_all("").unwrap();

    let mut parsed_path = String::new();

    let mut description: Option<String> = None;
    let mut title: Option<String> = None;
    let mut date: Option<String> = None;
    let mut file: Option<fs::File> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut write = false;
    for line in read_to_string(filename).unwrap().lines() {
        let s = line.to_string();
        // println!("{}",s);

        if s.contains("staticPath:") {
            if let Some(path_part) = s.splitn(2, ':').nth(1) {
                let path_part_trimmed = path_part.trim();

                parsed_path = path_part_trimmed.to_string();

                let mut static_base = hugo_base_dir.clone();

                static_base.push_str("public/");
                static_base.push_str(&parsed_path);

                println!("created static folder {}", static_base);
                fs::create_dir_all(static_base.clone()).unwrap();

                let mut hugo_md_file = hugo_base_dir.clone();
                fs::create_dir_all(hugo_base_dir.clone() + "app/" + &parsed_path)
                    .expect("couldn't make dir");
                hugo_md_file.push_str("app/");
                hugo_md_file.push_str(&parsed_path);
                hugo_md_file.push_str("/page.mdx");
                println!("Creating hugo md file {}", hugo_md_file);
                file = Some(File::create(hugo_md_file.clone()).unwrap());
            }
        }
        if s.contains("Description:") {
            if let Some(parsed_description) = s.splitn(2, ':').nth(1) {
                let parsed_description: &str = parse_text(parsed_description);
                description = Some(parsed_description.to_string());
            }
        }

        if s.contains("Title:") {
            if let Some(parsed_description) = s.splitn(2, ':').nth(1) {
                let parsed_description: &str = parse_text(parsed_description);
                title = Some(parsed_description.to_string());
            }
        }

        if s.contains("Date:") {
            if let Some(parsed_description) = s.splitn(2, ':').nth(1) {
                date = Some(parsed_description.to_string());
            }
        }
        if s.contains(" - ") {
            if let Some(parsed_tag) = s.splitn(2, "-").nth(1) {
                let parsed_tag = &parsed_tag[1..];
                tags.push(parsed_tag.to_string());
            }
        }
        if s.contains("---") {
            if !file.is_none() {
                write = true;
                write_header(&file, &title, &date, &description, &tags);
            }
        }
        if s.contains("![[") {
            let imagename = s
                .split("[[")
                .nth(1)
                .expect("Couldn't parse")
                .split("]]")
                .nth(0)
                .unwrap();

            let mut attachment_base = obsidan_img_folder.clone();
            attachment_base.push_str(imagename);

            let img_hash = hash32(imagename);

            let mut dest_path = hugo_base_dir.clone();
            dest_path.push_str("public/");
            dest_path.push_str(&parsed_path);
            dest_path.push_str("/");
            dest_path.push_str(&parsed_path);
            dest_path.push_str(&img_hash.to_string());
            dest_path.push_str(".png");

            println!("copied {} to {}", attachment_base, dest_path);

            fs::copy(attachment_base.clone(), dest_path.clone()).unwrap();

            let new_image_markdown: String = "![img](/".to_owned()
                + &parsed_path
                + "/"
                + &parsed_path
                + &img_hash.to_string()
                + ".png)";

            match file {
                Some(ref file) =>write_to_file(file, new_image_markdown),
                None => panic!("Trying to write image markdown to file without declaring `staticPath` have you set in the obsidian file?"),
            }
        } else {
            if write && !s.contains("---") {
                match file {
                Some(ref file) =>write_to_file(file, s),
                None => panic!("Trying to text to file without declaring `staticPath` have you set in the obsidian file?"),
            }
            }
        }
    }
}

fn write_to_file(mut file: &File, str: String) {
    file.write((str + "\n").as_bytes()).unwrap();
}

fn parse_text(s: &str) -> &str {
    let mut s = &s[1..];

    if s.contains("\"") {
        //trim qoutes
        s = &s[1..];
        s = &s[0..s.len() - 1];
    }

    return s;
}

fn write_header(
    file: &Option<File>,
    title: &Option<String>,
    _date: &Option<String>,
    _description: &Option<String>,
    _tags: &Vec<String>,
) {
    let mut file = file.as_ref().unwrap();
    let title = "# ".to_owned() + title.as_ref().expect("Title: Not set") + "\n";
    file.write(title.as_bytes()).unwrap();
}

fn publish_blag(bucket_name:String, distribution_id:String) {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_secs().to_string();
    let invalidation_cmd = format!(r#"Paths={{Quantity=1,Items=["/*"]}},CallerReference="{}""#,since_the_epoch);
    Command::new("npm")
        .current_dir("emma.rs")
        .arg("run")
        .arg("build")
        .output()
        .expect("npm build failed");
    println!("Ran npm build");
    Command::new("aws")
        .arg("rm")
        .arg("s3://".to_owned()+ &bucket_name)
        .arg("--recursive")
        .output()
        .expect("Couldn't clean bucket");
    println!("Cleaned bucket");
    Command::new("aws")
        .arg("sync")
        .arg("emma.rs/out/")
        .arg("s3://".to_owned() + &bucket_name)
        .output()
        .expect("Couldn't upload to bucket");
    println!("Synced to bucket");

    let invalidation_id =     Command::new("aws")
    .arg("cloudfront")
    .arg("create-invalidation")
    .arg("--distribution-id")
    .arg(&distribution_id)
    .arg("--invalidation-batch")
    .arg(invalidation_cmd)
    .arg("--query")
    .arg("Invalidation.Id")
    .arg("--output")
    .arg("text")
    .output()
    .expect("Couldn't create invalidation");    

    let invalidation_id =str::from_utf8(&invalidation_id.stdout).unwrap().trim();

    println!("Waiting on invalidation: {invalidation_id}");
    //doesn't work ://
    let wait_out = Command::new("aws")
    .arg("cloudfront")
    .arg("wait")
    .arg("invalidation-completed")
    .arg("--distribution-id")
    .arg(&distribution_id)
    .arg("--id")
    .arg(invalidation_id)
    .output()
    .expect("Couldn't wait on invalidation");

    println!("Done :)");
   
}

fn copy_includes() {

    let paths = fs::read_dir("emma.rs/app-includes").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let path = path.display().to_string();
        let dest = path.replace("app-includes", "app");
        println!("Copying {path} to {dest}");
        fs::copy(path, dest).unwrap();
    }

    let paths = fs::read_dir("emma.rs/public-includes").unwrap();
    for path in paths {
        let path = path.unwrap().path();
        let path = path.display().to_string();
        let dest = path.replace("public-includes", "public");
        println!("Copying {path} to {dest}");
        fs::copy(path, dest).unwrap();
    }
}
fn main() {
    dotenvy::dotenv().unwrap();


    let paths = fs::read_dir("blag-src/Publish").unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let display = path.display();

        process_blog(
            &("emma.rs/".to_string()),
            &("blag-src/Imgs/").to_string(),
            &display.to_string(),
            &("blag-src/Publish".to_string()),
        );
    }

    let bucket_name = env::var("BUCKET_NAME").expect("Couldn't load bucket name");
    let distrbition_id = env::var("DISTRIBUTION_ID").expect("couldn't load distribution id");

    copy_includes();
    publish_blag(bucket_name,distrbition_id);
}
