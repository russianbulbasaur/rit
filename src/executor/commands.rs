use std::fs;
use std::fs::{File, FileType};
use std::io::{BufWriter, Write, BufReader, Read, BufRead, Cursor};
use std::path::Path;
use sha1::{Digest, Sha1};
use miniz_oxide::deflate::{compress_to_vec_zlib};
use miniz_oxide::inflate::{decompress_to_vec_zlib};


struct Blob{
    size:usize,
    content:Vec<u8>
}

struct TreeEntry{
    entry_type:TreeEntryType,
    hash:String,
    name:String,
    mode:u64,
}

enum TreeEntryType{
    BLOB,
    TREE
}

struct Tree{
    entries:Vec<TreeEntry>
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


pub fn hash_object(arguments:Vec<String>) -> String{
    if arguments[2] != "-w" {
        println!("Invalid {} flag",arguments[2]);
        return String::new();
    }
    let source = arguments[3].clone();
    let mut source_file = File::open(source).unwrap();
    let mut file_contents = Vec::<u8>::new();
    let n = source_file.read_to_end(&mut file_contents).unwrap();
    let mut sha_hash_gen = Sha1::new();
    sha_hash_gen.update(&file_contents);
    let output = sha_hash_gen.finalize();
    let hash = format!("{output:x}");
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
    hash
}


pub fn ls_tree(arguments:Vec<String>) {
    let mut name_only = false;
    let mut file_hash= String::from("");
    if arguments.len() > 3 {
        if arguments[2] == "--name-only"{
            file_hash = arguments[3].clone();
            name_only = true;
        }else {
            println!("Invalid argument {}",arguments[2]);
        }
    }else{
        file_hash = arguments[2].clone();
    }

    let dir_name = &file_hash[0..2];
    let file_name = &file_hash[2..];
    let file_path = format!(".git/objects/{dir_name}/{file_name}");
    let tree = read_tree(Path::new(&file_path));
    for entry in tree.entries {
        if name_only {
            println!("{}", entry.name);
        }else{
            println!("{}   {}   {}",entry.mode,entry.name,entry.hash);
        }
    }
}

pub fn write_tree(arguments:Vec<String>,directory:String) -> String{
    let mut tree_entries = Vec::<TreeEntry>::new();
    let directory_entries = fs::read_dir(directory).unwrap();
    for entry in directory_entries{
        let directory_entry = entry.unwrap();
        let path = directory_entry.path().into_os_string().into_string().unwrap();
        let name = directory_entry.file_name().into_string().unwrap();
        if name == ".git" {
            continue;
        }
        let file_type = directory_entry.file_type().unwrap();
        if file_type.is_file(){
            //create a blob
            let blob_hash = hash_object(
                vec!["".to_string(),"".to_string(),"-w".to_string(),
                path]);
            if blob_hash.is_empty(){
                continue;
            }
            tree_entries.push(
                TreeEntry{
                    entry_type:TreeEntryType::BLOB,
                    hash:blob_hash,
                    name,
                    mode:1040000
                }
            );
        }else if file_type.is_dir(){
            //create a tree and record hash
            let tree_hash = write_tree(Vec::new(),path);
            if tree_hash.is_empty(){
                continue;
            }
            tree_entries.push(
                TreeEntry{
                    entry_type:TreeEntryType::TREE,
                    mode:40000,
                    name,
                    hash:tree_hash
                }
            );
        }
    }
    let tree_hash = write_and_compress_tree_object(tree_entries);
    tree_hash
}



fn write_and_compress_tree_object(entries:Vec<TreeEntry>) -> String{
    let mut write_buffer = Vec::<u8>::new();
    let mut n;

    //add entries
    n = write_buffer.write("tree 20".as_bytes()).unwrap();
    n = write_buffer.write(&[b'\0']).unwrap();

    for entry in entries{
        let mode_string = entry.mode.to_string();
        let name = entry.name;
        let hash = entry.hash;
        let to_be_written = format!("{mode_string} {name}");
        write_buffer.write(to_be_written.as_bytes()).unwrap();
        write_buffer.write(&[b'\0']).unwrap();
        write_buffer.write(hash.as_bytes()).unwrap();
    }

    let mut hasher = Sha1::new();
    hasher.update(&write_buffer);
    let hasher_output = hasher.finalize();
    let hash = format!("{hasher_output:x}");

    let dir_name = &hash[0..2];
    let file_name = &hash[2..];
    let dir_path = format!(".git/objects/{dir_name}");
    fs::create_dir(dir_path).unwrap();
    let file_path = format!(".git/objects/{dir_name}/{file_name}");
    let tree_file = File::create(file_path).unwrap();
    let mut buf_writer = BufWriter::new(tree_file);

    let compressed_tree_contents = compress_to_vec_zlib(&write_buffer,6);
    buf_writer.write(&compressed_tree_contents).unwrap();
    hash
}

fn read_tree(path:&Path) -> Tree {
    let mut tree_file = File::open(path).unwrap();
    let mut n;
    let mut compressed_tree_file_contents : Vec<u8> = Vec::new();
    tree_file.read_to_end(&mut compressed_tree_file_contents).unwrap();
    let decompressed_tree_file_contents = decompress_to_vec_zlib(&compressed_tree_file_contents).unwrap();
    let file_contents_cursor = Cursor::new(decompressed_tree_file_contents);
    let mut buf_reader = BufReader::new(file_contents_cursor);

    //marker
    let mut marker_buffer = Vec::<u8>::new();
    n = buf_reader.read_until(b' ',&mut marker_buffer).unwrap();
    marker_buffer.remove(n-1);
    let marker = String::from_utf8(marker_buffer).unwrap();
    if marker != "tree" {
        panic!("ls-tree called on a non tree like file");
    }


    //size
    let mut size_buffer = Vec::<u8>::new();
    n = buf_reader.read_until(b'\0',&mut size_buffer).unwrap();
    size_buffer.remove(n-1); //\0
    let tree_size = String::from_utf8(size_buffer).unwrap().parse().unwrap();

    //reading entries
    let mut tree_entries = Vec::<TreeEntry>::new();
    let mut bytes_read = 0;
    while bytes_read<tree_size{
        let mut entry_mode_buffer = Vec::new();
        n = buf_reader.read_until(b' ',&mut entry_mode_buffer).unwrap();
        bytes_read += n;
        entry_mode_buffer.remove(n-1); //empty space
        let mode = String::from_utf8(entry_mode_buffer).unwrap().parse().unwrap();
        let mut entry_type : TreeEntryType = TreeEntryType::BLOB;
        if mode == 40000 {
            entry_type = TreeEntryType::TREE;
        }

        let mut name_buffer = Vec::new();
        n = buf_reader.read_until(b'\0',&mut name_buffer).unwrap();
        bytes_read += n;
        name_buffer.remove(n-1); //\0
        let name = String::from_utf8(name_buffer).unwrap();

        let mut hash_buffer:[u8;20] = [0;20];
        buf_reader.read_exact(&mut hash_buffer).unwrap();
        bytes_read += 20;
        let mut hash = hex::encode(&hash_buffer);

        tree_entries.push(TreeEntry{
            mode,
            entry_type,
            name,
            hash,
        });
    }

    Tree{
        entries:tree_entries
    }
}




fn read_blob(path:&Path) -> Blob{
    let mut blob_file = File::open(path).unwrap();
    let mut compressed_file_contents = Vec::<u8>::new();
    blob_file.read_to_end(&mut compressed_file_contents).unwrap();
    let decompressed_file_contents = decompress_to_vec_zlib(&compressed_file_contents).unwrap();
    let mut buf_reader = BufReader::new(Cursor::new(decompressed_file_contents));

    //marker
    let mut decoded_marker_buffer = Vec::<u8>::new();
    let n = buf_reader.read_until(b' ',&mut decoded_marker_buffer).unwrap();
    decoded_marker_buffer.remove(n-1);


    //size
    let mut decoded_size_buffer = Vec::<u8>::new();
    let n = buf_reader.read_until(b'\0',&mut decoded_size_buffer).unwrap();
    decoded_size_buffer.remove(n-1);

    //content
    let mut decoded_content_buffer = Vec::<u8>::new();
    let n = buf_reader.read_to_end(&mut decoded_content_buffer).unwrap();
    Blob{
        size : String::from_utf8(decoded_size_buffer).unwrap().parse().unwrap(),
        content: decoded_content_buffer
    }
}


fn write_blob(file:File,blob:Blob){
    let mut buf_writer = BufWriter::new(file);
    let mut content_vector = Vec::<u8>::new();
    content_vector.write("blob ".as_ref()).unwrap();
    content_vector.write(blob.size.to_string().as_ref()).unwrap();
    content_vector.write(&[b'\0']).unwrap();
    content_vector.write(&blob.content[0..]).unwrap();
    let compressed_content_vector = compress_to_vec_zlib(&content_vector,6);
    buf_writer.write(&compressed_content_vector).unwrap();
}