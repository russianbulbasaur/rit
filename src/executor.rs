mod commands;

pub fn execute(arguments:Vec<String>){
    match arguments[1].as_str(){
        "init" => commands::init(),
        "cat-file" => commands::cat_file(arguments),
        "hash-object" => {
            commands::hash_object(arguments);
        },
        "ls-tree" => commands::ls_tree(arguments),
        "write-tree" => {
            commands::write_tree(arguments,String::from("."));
        },
        _ => println!("Command not implemented yet")
    }
}

