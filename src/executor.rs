mod commands;

pub fn execute(arguments:Vec<String>){
    println!("{}",arguments[1]);
    match arguments[1].as_str(){
        "init" => commands::init(),
        "cat-file" => commands::cat_file(arguments),
        "hash-object" => commands::hash_object(arguments),
        _ => println!("Command not implemented yet")
    }
}

