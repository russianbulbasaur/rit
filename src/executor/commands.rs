use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write, BufReader, Read, BufRead};
use std::path::Path;


struct Blob{
    size:usize,
    content:Vec<u8>
}

pub fn init(){
    fs::create_dir_all(".git/objects").unwrap();
    fs::create_dir_all(".git/refs").unwrap();
    let head_file = File::create(".git/HEAD").unwrap();
    let mut buf_writer = BufWriter::new(head_file);
    let init_entry = "ref: refs/heads/main\n";
    let n = buf_writer.write(init_entry.as_ref()).unwrap();
    println!("Wrote {} bytes into head",n);
    buf_writer.flush().expect("unable to flush");
}


pub fn cat_file(arguments:Vec<String>){
    if arguments[2] != "-p" {
        println!("Unexpected argument {}",arguments[2]);
        return;
    }
    let blob_sha = arguments[3].clone();
    println!("SHA {}",blob_sha);
    let sha_dir = &blob_sha[0..2];
    let file_name = &blob_sha[2..];
    let str_path = format!(".git/objects/{sha_dir}/{file_name}");
    let path = Path::new(str_path.as_str());
    let blob = read_blob(path);
    println!("size {}",blob.size);
    println!("content {}",String::from_utf8(blob.content).unwrap());
}


pub fn hash_object(arguments:Vec<String>){
    if arguments[2] != "-w" {
        println!("Invalid {} flag",arguments[2]);
        return
    }
    let source = arguments[3].clone();
    let mut source_file = File::open(source).unwrap();
    let mut file_contents = Vec::<u8>::new();
    let n = source_file.read_to_end(&mut file_contents).unwrap();

    let hash = "hashhehe";
    let hash_dir = &hash[0..2];
    let hash_dir_path = format!(".git/objects/{hash_dir}");
    fs::create_dir(hash_dir_path).unwrap();
    let hash_file = &hash[2..];
    let hash_file_path = format!(".git/objects/{hash_dir}/{hash_file}");
    let hashed_file = File::create(hash_file_path).unwrap();
    write_blob(hashed_file,Blob{
        size:n,
        content:file_contents,
    });
    println!("{}",hash);
}

fn read_blob(path:&Path) -> Blob{
    let blob_file = File::open(path).unwrap();
    let mut buf_reader = BufReader::new(blob_file);
    let mut marker_buffer = Vec::<u8>::new();
    let n = buf_reader.read_until(b' ',&mut marker_buffer).unwrap();
    marker_buffer.remove(marker_buffer.len()-1);
    let mut size_buffer = Vec::<u8>::new();
    let n = buf_reader.read_until(b'\0',&mut size_buffer).unwrap();
    size_buffer.remove(size_buffer.len()-1);
    let mut content_buffer = Vec::<u8>::new();
    let n = buf_reader.read_to_end(&mut content_buffer).unwrap();
    Blob{
        size : String::from_utf8(size_buffer).unwrap().parse().unwrap(),
        content: content_buffer
    }
}


fn write_blob(file:File,blob:Blob){
    let mut buf_writer = BufWriter::new(file);
    buf_writer.write("blob ".as_ref()).unwrap();
    buf_writer.write(blob.size.to_string().as_ref()).unwrap();
    buf_writer.write(&[b'\0']).unwrap();
    buf_writer.write(&blob.content[0..]).unwrap();
}