use chrono::Datelike;
use chrono::NaiveTime;
use fxhash::hash32;
use std::collections::BTreeMap;
use std::fs;
use std::fs::read_to_string;
use std::fs::File;
use std::io::prelude::*;
use std::process::exit;
use std::process::Command;
use std::process::Output;
use std::str;
use dotenvy;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::Path;
use std::io;
use chrono::NaiveDate;
use chrono::Local;
use chrono_tz::US::Mountain;
use chrono::TimeZone;


#[derive(Debug,Clone)]
struct Article {
    title: String,
    date: String,
    path: String,
    description: String,
    tags: Vec<String>
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}


fn process_blog(
    hugo_base_dir: &String,
    obsidan_img_folder: &String,
    filename: &str,
    _obsidan_dir: &String,
) -> Article {
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
            if !write {
                if let Some(parsed_tag) = s.splitn(2, "-").nth(1) {
                    let parsed_tag = &parsed_tag[1..];
                    tags.push(parsed_tag.to_string());
                }
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

    Article {
        title: title.unwrap(),
        date: date.unwrap(),
        path: parsed_path,
        description: description.unwrap(),
        tags
    }
}

fn write_to_file(mut file: &File, str: String) {
    file.write((str + "  \n").as_bytes()).unwrap();
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

fn handle_out(out: Output) {
    println!("{}",str::from_utf8(&out.stdout).unwrap());
    if out.stderr.len() != 0 {
        println!("len {}",out.stderr.len());
        println!("stderr: {}",str::from_utf8(&out.stderr).unwrap());
       // exit(69);
    }
}
fn publish_blag(bucket_name:String, distribution_id:String) {
    let bucket_name = bucket_name.trim();
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards").as_secs().to_string();
    let invalidation_cmd = format!(r#"Paths={{Quantity=1,Items=["/*"]}},CallerReference="{}""#,since_the_epoch);
    
    let out = Command::new("npm")
        .current_dir("emma.rs")
        .arg("run")
        .arg("build")
        .output()
        .expect("npm build failed");
    handle_out(out);
    println!("Ran npm build");
    let out = Command::new("aws")
        .arg("s3")
        .arg("rm")
        .arg("s3://".to_owned()+ &bucket_name)
        .arg("--recursive")
        .output()
        .expect("Couldn't clean bucket");
    handle_out(out);

   
    
    println!("Cleaned bucket");
    let out = Command::new("aws")
        .arg("s3")
        .arg("sync")
        .arg("emma.rs/out/")
        .arg("s3://".to_owned() + &bucket_name)
        .output()
        .expect("Couldn't upload to bucket");
    handle_out(out);
    

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
    handle_out(invalidation_id.clone());

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

    if wait_out.stdout.len() != 0 {
        println!("{}",str::from_utf8(&wait_out.stdout).unwrap());
        exit(69);
    }

    println!("Done :)");
   
}



fn copy_includes(src_dir: &str,dest_dir:&str) {

    let src_path = format!("emma.rs/{}",src_dir);
    let paths = fs::read_dir(src_path).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        let path = path.display().to_string();
        let dest = path.replace(&src_dir, &dest_dir);
        let attr = fs::metadata(path.clone()).expect("couldn't get metadata");
        if attr.is_file() {
            println!("Copying {path} to {dest}");
            fs::copy(path, dest).unwrap();
        } else if attr.is_dir() {
            println!("Copying dir {path} to {dest}");
            copy_dir_all(path, dest).expect("coudn't copy dir");
        }
    }
}

fn cleanup_pre_build() {
    fs::remove_dir_all("emma.rs/public").expect("couldn't remove public dir");
    fs::remove_dir_all("emma.rs/app").expect("Couldn't remove app dir");

    fs::create_dir("emma.rs/public").expect("Couldn't create public dir");
    fs::create_dir("emma.rs/app").expect("Couldn't create app dir");
    println!("Done prebuild clean up");
}

fn cool_date(date: String) -> String {
    let date = NaiveDate::parse_from_str(&date,"%Y-%m-%d").unwrap();
    let month = &date.format("%B").to_string()[0..3];
    let year = date.year().to_string();

    format!("{} {}",month,year)



}
fn convert_obs_date_to_rss_date(date: &String) -> String {
    let date = NaiveDate::parse_from_str(date,"%Y-%m-%d").unwrap();
    let noon = NaiveTime::from_hms_opt(12,0,0).unwrap();
    let datetime = date.and_time(noon);
    let mountain_time = Mountain.from_local_datetime(&datetime).unwrap();

    mountain_time.format("%a, %-d %b %Y %H:%M:%S %Z").to_string()
}

fn write_article( w:&mut Vec<u8>,link: &Article) {
    writeln!( w, "\t<item>").unwrap();
    writeln!( w,"\t\t<title>{}</title>",link.title).unwrap();
    writeln!( w,"\t\t<description>{}</description>",link.description).unwrap();
    let url = format!("https://emma.rs/{}",link.path);
    writeln!( w,"\t\t<link>{}</link>",url).unwrap();
    let guid = hash32(&url);
    writeln!( w,"\t\t<guid>{}</guid>",guid).unwrap();
    let date = convert_obs_date_to_rss_date(&link.date);
    writeln!( w,"\t\t<pubDate>{}</pubDate>",date).unwrap();
    writeln!( w, "\t</item>").unwrap();
}
fn gen_rss(links:& Vec<Article>) {
    let now = Local::now();
    let year = now.format("%Y").to_string();

    let build_date = now.format("%a, %-d %b %Y %H:%M:%S %z").to_string();
    let mut w = Vec::new();
    writeln!(&mut w,"<?xml version=\"1.0\" encoding=\"UTF-8\" ?>").unwrap();
    writeln!(&mut w,"<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">").unwrap();
    writeln!(&mut w,"<channel>").unwrap();
    writeln!(&mut w,"<atom:link href=\"https://emma.rs/rss.xml\" rel=\"self\" type=\"application/rss+xml\" />").unwrap();
    writeln!(&mut w,"\t<title>emma.rs feed</title>").unwrap();
    writeln!(&mut w,"\t<description>This is a feed for emma.rs. Focusing on malware reverse engineering, vuln research and browser fuzzing</description>").unwrap();
    writeln!(&mut w,"\t<link>https://emma.rs</link>").unwrap();
    writeln!(&mut w,"\t<copyright>{} emma.rs All rights reserved</copyright>",year).unwrap();
    writeln!(&mut w,"\t<lastBuildDate>{}</lastBuildDate>",build_date).unwrap();
    writeln!(&mut w,"\t<pubDate>{}</pubDate>",build_date).unwrap();
    writeln!(&mut w,"\t<ttl>1800</ttl>").unwrap();
    
    for link in links {
        write_article(&mut w, link);
    }
    writeln!(&mut w).unwrap();
    writeln!(&mut w, "</channel>").unwrap();
    writeln!(&mut w,"</rss>").unwrap();

    fs::write("emma.rs/public/rss.xml", w).expect("Unable to write xml file");


}

fn gen_tags(links:&mut Vec<Article>) {

    let mut tags = BTreeMap::new();


    for link in links {
        for tag in link.tags.clone().into_iter() {
            if !tags.contains_key(&tag) {
                tags.insert(tag,vec![link.clone()] );
            } else {
                let list = tags.get_mut(&tag).unwrap();
                list.push(link.clone());
            }
            
        }
    }
    let mut w = Vec::new();
    for (tag_name,articles) in tags.iter() {
        writeln!(&mut w,"# {}",tag_name).unwrap();
        for article in articles {
            writeln!(&mut w,"- [{}]({})",article.title,article.path).unwrap();
        }

        writeln!(&mut w).unwrap();
        writeln!(&mut w).unwrap();
    }

    fs::create_dir_all("emma.rs/app/tags/").expect("coudln't create tags dir");
    fs::write("emma.rs/app/tags/page.mdx", w).expect("Unable to write tags file");

}

fn write_main_page(links:&mut Vec<Article>) {
    //order by date
    links.sort_by(|a,b| b.date.partial_cmp(&a.date).unwrap());
    let mut w = Vec::new();
    writeln!(&mut w,"# Hi! 🦑").unwrap();
    writeln!(&mut w).unwrap();
    writeln!(&mut w).unwrap();
    writeln!(&mut w,"I’m Emma (They/Them). My interests are malware reversing, penetration testing, and browser fuzzing.").unwrap();
    writeln!(&mut w).unwrap();
    writeln!(&mut w,"<p class=\"highlight\">Loading...</p>").unwrap();
    writeln!(&mut w,"<script src=\"static/wasm.js\"></script>").unwrap();
    for link in links {
        //println!("{:#?}",link);
        let date = cool_date(link.date.clone());
        writeln!(&mut w, "- [{}]({}) ({})  ",link.title,link.path, date).unwrap();
    }    
    writeln!(&mut w, "[Explore Tags](/tags)").unwrap();   

    writeln!(&mut w).unwrap();
    writeln!(&mut w,"Subscribe to my [RSS Feed](https://emma.rs/rss.xml)").unwrap();
    fs::write("emma.rs/app/page.mdx", w).expect("Unable to write main page");
}

fn main() {
    dotenvy::dotenv().unwrap();

    cleanup_pre_build();
    copy_includes("app-includes","app");
    copy_includes("public-includes", "public");
    let paths = fs::read_dir("blag-src/Publish").unwrap();

    let mut links = vec![];
    for path in paths {
        let path = path.unwrap().path();
        let display = path.display();

        let link = process_blog(
            &("emma.rs/".to_string()),
            &("blag-src/Imgs/").to_string(),
            &display.to_string(),
            &("blag-src/Publish".to_string()),
        );
        links.push(link);
    }


    let bucket_name = env::var("BUCKET_NAME").expect("Couldn't load bucket name");
    let distribution_id = env::var("DISTRIBUTION_ID").expect("couldn't load distribution id");


    write_main_page(&mut links);
    println!("Wrote main page");
    gen_rss(&links);
    println!("Generated rss feed");
    gen_tags(&mut links);
    println!("Generated tags page");
    publish_blag(bucket_name,distribution_id);
}
