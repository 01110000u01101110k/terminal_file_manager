use crossterm::execute;
use crossterm::cursor::MoveTo;
use crossterm::event::EnableMouseCapture;
use crossterm::event::DisableMouseCapture;
use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;

use chrono::DateTime;
use chrono::offset::Local;

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use std::fs;
use std::{io::Result, io::stdout, io::Stdout};
use std::io::{self, Write};
use std::path::PathBuf;
use std::io::stdin;
use std::fs::File;
use std::fs::create_dir;
use std::fs::remove_dir;
use std::fs::remove_file;
use std::fs::metadata;
use std::thread::sleep;
use std::time::Duration;

use crate::context_menu::{ContextMenuItems};
use crate::target_directory::{TargetDirectory};
use crate::clear::{clear};

pub fn enter(stdout: &mut Stdout, target_directory: &mut TargetDirectory) -> Result<()> {
    if target_directory.context_menu.is_open_menu {
        match target_directory.context_menu.menu[target_directory.context_menu.selected] {
            ContextMenuItems::DeleteItem => {
                execute!(
                    stdout,
                    MoveTo(0, 0),
                    EnterAlternateScreen,
                    DisableMouseCapture,
                    Show
                ).unwrap();

                let mut dir = fs::read_dir(&target_directory.path).unwrap();

                let dir_item = dir.nth(target_directory.selected).unwrap().unwrap();

                if dir_item.path().is_dir() {
                    print!("ви дійсно хочете видалити дерикторію {}? Введіть 'yes' або 'no'.", &dir_item.file_name().to_str().unwrap());
                    std::io::stdout().flush().expect("Помилка при очищенні буфера");

                    let mut entered = String::new();
                    stdin().read_line(&mut entered).expect("something went wrong when writing the name of directory");

                    if entered.trim() == "yes" || entered.trim() == "y" {
                        let dir = fs::read_dir(&dir_item.path()).unwrap();
                        
                        let mut folder_hierarchy: Vec<(i32, PathBuf)> = Vec::new();

                        let position_in_hierarchy = 0;

                        find_folders_and_delete_files_inside(dir, position_in_hierarchy, &mut folder_hierarchy);

                        fn find_folders_and_delete_files_inside(dir_content: fs::ReadDir, position_in_hierarchy: i32, folder_hierarchy: &mut Vec<(i32, PathBuf)>) {
                            for dir_item in dir_content {
                                match dir_item {
                                    Ok(dir_item) => {
                                        if dir_item.path().is_dir() {
                                            folder_hierarchy.push((position_in_hierarchy, dir_item.path()));

                                            match fs::read_dir(dir_item.path()) {
                                                Ok(item) => {
                                                    find_folders_and_delete_files_inside(item, position_in_hierarchy + 1, folder_hierarchy);
                                                },
                                                Err(error) => {
                                                    eprintln!("error: {}", error);
                                                }
                                            }
                                        } else {
                                            let remove_file = remove_file(dir_item.path().to_str().unwrap());

                                            match remove_file {
                                                Ok(_) => {
                                                    println!("файл {} був успішно видалений", &dir_item.file_name().to_str().unwrap());
                                                },
                                                Err(error) => {
                                                    eprintln!("error: {}", error);
                                                }
                                            }
                                        }
                                    },
                                    Err(error) => {
                                        eprintln!("error: {}", error);
                                    }
                                }
                            }
                        }

                        folder_hierarchy.sort_by(|a, b| b.0.cmp(&a.0));

                        folder_hierarchy.into_iter().for_each(|item| {
                            let remove_dir = remove_dir(&item.1);

                            match remove_dir {
                                Ok(_) => {
                                    println!("папка {} була видалена", &item.1.to_str().unwrap());
                                },
                                Err(error) => {
                                    eprintln!("папка {} не була видалена, по причині: {}", &item.1.to_str().unwrap(), error);
                                }
                            }
                        });

                        let remove_main_selected_dir = remove_dir(&dir_item.path());

                        match remove_main_selected_dir {
                            Ok(_) => {
                                println!("папка {} була видалена", &dir_item.path().to_str().unwrap());
                            },
                            Err(error) => {
                                eprintln!("папка {} не була видалена, по причині: {}", &dir_item.path().to_str().unwrap(), error);
                            }
                        }

                        sleep(Duration::from_millis(2000));
                    }
                } else {
                    print!("ви дійсно хочете видалити файл {}? Введіть 'yes' або 'no'.", &dir_item.file_name().to_str().unwrap());
                    std::io::stdout().flush().expect("Помилка при очищенні буфера");

                    let mut entered = String::new();
                    stdin().read_line(&mut entered).expect("something went wrong when writing the name of directory");
                    
                    if entered.trim() == "yes" || entered.trim() == "y" {
                        let remove_file = remove_file(dir_item.path().to_str().unwrap());

                        match remove_file {
                            Ok(_) => {
                                clear(stdout);
                                println!("файл {} був успішно видалений", &dir_item.file_name().to_str().unwrap());
                            },
                            Err(error) => {
                                clear(stdout);
                                eprintln!("error: {}", error);
                            }
                        }

                        sleep(Duration::from_millis(2000));
                    }
                }

                target_directory.context_menu.is_open_menu = false;

                execute!(
                    stdout,
                    MoveTo(0, 0),
                    LeaveAlternateScreen,
                    EnableMouseCapture, 
                    Hide
                ).unwrap();

                clear(stdout);
                target_directory.print_dir_content();
            },
            ContextMenuItems::CreateDir => {
                execute!(
                    stdout,
                    /*SetStyle(
                        ContentStyle {
                            foreground_color: Some(Black),
                            background_color: Some(Green),
                            underline_color: None,
                            attributes: Attribute::Bold.into(),
                        }
                    ),*/
                    MoveTo(0, 0),
                    EnterAlternateScreen,
                    DisableMouseCapture,
                    Show
                ).unwrap();

                print!("enter the name of the directory you want to create: ");
                std::io::stdout().flush().expect("Помилка при очищенні буфера");

                let mut dir_name = String::new();
                stdin().read_line(&mut dir_name).expect("something went wrong when writing the name of directory");

                let mut patch_to_new_dir = target_directory.path.clone();

                patch_to_new_dir.push(&dir_name);

                let directory = create_dir(patch_to_new_dir.as_path().to_str().unwrap().trim())/*.expect(&format!("something went wrong when trying to create a new directory with the name {}", &dir_name))*/;
                
                match directory {
                    Ok(_) => {
                        println!("дерикторія {} була успішно створена", &dir_name.trim());
                    },
                    Err(error) => {
                        eprintln!("error: {}", error);
                    }
                }

                target_directory.context_menu.is_open_menu = false;

                sleep(Duration::from_millis(2000));

                execute!(
                    stdout,
                    MoveTo(0, 0),
                    LeaveAlternateScreen,
                    EnableMouseCapture, 
                    Hide
                ).unwrap();

                clear(stdout);
                target_directory.print_dir_content();
            },
            ContextMenuItems::CreateFile => {
                execute!(
                    stdout,
                    MoveTo(0, 0),
                    EnterAlternateScreen,
                    DisableMouseCapture,
                    Show
                ).unwrap();

                print!("enter the name of the file you want to create: ");
                std::io::stdout().flush().expect("Помилка при очищенні буфера");

                let mut file_name = String::new();
                stdin().read_line(&mut file_name).expect("something went wrong when writing the name of file");

                let mut patch_to_new_file = target_directory.path.clone();

                patch_to_new_file.push(&file_name);

                let file = File::create(&patch_to_new_file.as_path().to_str().unwrap().trim())/*.expect(&format!("something went wrong when trying to create a new file with the name {}", &file_name))*/;
                
                match file {
                    Ok(_) => {
                        println!("файл {} успішно створений", &file_name.trim());
                    },
                    Err(error) => {
                        eprintln!("error: {}", error);
                    }
                }

                target_directory.context_menu.is_open_menu = false;

                sleep(Duration::from_millis(2000));

                execute!(
                    stdout,
                    MoveTo(0, 0),
                    LeaveAlternateScreen,
                    EnableMouseCapture, 
                    Hide
                ).unwrap();

                clear(stdout);
                target_directory.print_dir_content();
            },
            ContextMenuItems::Copy => { // todo - винести в окрему функцію разом з кодом з гілки CutItem (код в обох гілках майже ідентичний)
                let mut dir = fs::read_dir(&target_directory.path).unwrap();

                let dir_item = dir.nth(target_directory.selected).unwrap().unwrap();

                let mut path = target_directory.path.clone();

                path.push(dir_item.path());
                
                target_directory.context_menu.remember_way_to_selected_item = Some(path);

                target_directory.context_menu.is_open_menu = false;

                clear(stdout);
                target_directory.print_dir_content();
            },
            ContextMenuItems::Paste => {
                let mut current_path = target_directory.path.clone();

                let way_to_selected_item = target_directory
                    .context_menu
                    .remember_way_to_selected_item
                    .clone()
                    .unwrap();

                if way_to_selected_item.is_dir() {
                    create_dir_with_content(current_path, way_to_selected_item.clone());

                    fn create_dir_with_content(current_path: PathBuf, way_to_selected_item: PathBuf) {
                        let mut current_path = current_path;
                        let last_item = way_to_selected_item
                            .iter()
                            .last()
                            .unwrap();

                        current_path.push(last_item);

                        let directory = create_dir(current_path.clone());

                        match directory {
                            Ok(_) => {
                                let dir_content = fs::read_dir(&way_to_selected_item).unwrap();

                                for dir_item in dir_content {
                                    match dir_item {
                                        Ok(item) => {
                                            if item.path().is_dir() {
                                                create_dir_with_content(current_path.clone(), item.path());
                                            } else {
                                                let mut current_path = current_path.clone();

                                                current_path.push(
                                                    item.path()
                                                        .iter()
                                                        .last()
                                                        .unwrap()
                                                );

                                                let copy_file = fs::copy(&item.path(), &current_path);

                                                match copy_file {
                                                    Ok(status) => {
                                                        println!("файл {}, був успішно скопійований", &item.path().to_str().unwrap());
                                                    },
                                                    Err(error) => {
                                                        eprintln!("error: {}", error);
                                                    }
                                                }
                                            }
                                        },
                                        Err(error) => {
                                            eprintln!("error: {}", error);
                                        }
                                    }
                                }
                            },
                            Err(error) => {
                                eprintln!("error: {}", error);
                            }
                        }
                    }
                } else {
                    let last_item = way_to_selected_item
                        .iter()
                        .last()
                        .unwrap();

                    current_path.push(last_item);

                    let copy_file = fs::copy(&way_to_selected_item, &current_path);

                    match copy_file {
                        Ok(status) => {
                            println!("copy_file status: {}", status);
                        },
                        Err(error) => {
                            eprintln!("error: {}", error);
                        }
                    }
                }

                if target_directory.context_menu.need_to_cut_elmenet {
                    if way_to_selected_item.is_dir() {
                        let dir_content = fs::read_dir(&way_to_selected_item).unwrap();

                        let mut folder_hierarchy: Vec<(i32, PathBuf)> = Vec::new();

                        let position_in_hierarchy = 0;

                        find_folders_and_delete_files_inside(dir_content, position_in_hierarchy, &mut folder_hierarchy);

                        fn find_folders_and_delete_files_inside(dir_content: fs::ReadDir, position_in_hierarchy: i32, folder_hierarchy: &mut Vec<(i32, PathBuf)>) {
                            for dir_item in dir_content {
                                match dir_item {
                                    Ok(dir_item) => {
                                        if dir_item.path().is_dir() {
                                            folder_hierarchy.push((position_in_hierarchy, dir_item.path()));

                                            match fs::read_dir(dir_item.path()) {
                                                Ok(item) => {
                                                    find_folders_and_delete_files_inside(item, position_in_hierarchy + 1, folder_hierarchy);
                                                },
                                                Err(error) => {
                                                    eprintln!("error: {}", error);
                                                }
                                            }
                                        } else {
                                            let remove_file = remove_file(dir_item.path().to_str().unwrap());

                                            match remove_file {
                                                Ok(_) => {
                                                    println!("файл {} був успішно видалений", &dir_item.file_name().to_str().unwrap());
                                                },
                                                Err(error) => {
                                                    eprintln!("error: {}", error);
                                                }
                                            }
                                        }
                                    },
                                    Err(error) => {
                                        eprintln!("error: {}", error);
                                    }
                                }
                            }
                        }

                        folder_hierarchy.sort_by(|a, b| b.0.cmp(&a.0));

                        folder_hierarchy.into_iter().for_each(|item| {
                            let remove_dir = remove_dir(&item.1);

                            match remove_dir {
                                Ok(_) => {
                                    println!("папка {} була видалена", &item.1.to_str().unwrap());
                                },
                                Err(error) => {
                                    eprintln!("папка {} не була видалена, по причині: {}", &item.1.to_str().unwrap(), error);
                                }
                            }
                        });

                        let remove_main_selected_dir = remove_dir(&way_to_selected_item);

                        match remove_main_selected_dir {
                            Ok(_) => {
                                println!("папка {} була видалена", way_to_selected_item.to_str().unwrap());
                            },
                            Err(error) => {
                                eprintln!("папка {} не була видалена, по причині: {}", way_to_selected_item.to_str().unwrap(), error);
                            }
                        }

                    } else {
                        let remove_file = remove_file(&way_to_selected_item);
                    
                        match remove_file {
                            Ok(_) => {
                                println!("файл {} був видалений", way_to_selected_item.to_str().unwrap());
                            }, 
                            Err(error) => {
                                eprintln!("файл {} не був видалений, по причині: {}", way_to_selected_item.to_str().unwrap(), error);
                            }
                        }
                    }

                    target_directory.context_menu.need_to_cut_elmenet = false;
                }

                target_directory.context_menu.is_open_menu = false;

                sleep(Duration::from_millis(5000));

                clear(stdout);
                target_directory.print_dir_content();
            },
            ContextMenuItems::CutItem => {
                let mut dir = fs::read_dir(&target_directory.path).unwrap();

                let dir_item = dir.nth(target_directory.selected).unwrap().unwrap();

                let mut path = target_directory.path.clone();

                path.push(dir_item.path());
                
                target_directory.context_menu.remember_way_to_selected_item = Some(path);

                target_directory.context_menu.need_to_cut_elmenet = true;

                target_directory.context_menu.is_open_menu = false;

                clear(stdout);
                target_directory.print_dir_content();
            },
            ContextMenuItems::Rename => {
                execute!(
                    stdout,
                    MoveTo(0, 0),
                    EnterAlternateScreen,
                    DisableMouseCapture,
                    Show
                ).unwrap();

                let mut dir = fs::read_dir(&target_directory.path).unwrap();

                let dir_item = dir.nth(target_directory.selected).unwrap().unwrap();

                let mut file = target_directory.path.clone();

                file.push(dir_item.path());

                print!("enter a new name: ");
                std::io::stdout().flush().expect("Помилка при очищенні буфера");

                let mut file_name = String::new();
                stdin().read_line(&mut file_name).expect("something went wrong when writing the name of file");

                let mut file_with_new_name = target_directory.path.clone();

                file_with_new_name.push(&file_name.trim());

                let rename_result = fs::rename(&file, file_with_new_name);

                match rename_result {
                    Ok(_) => {
                        println!("файл {} успішно перейменований на {}", file.display(), file_name);
                    },
                    Err(error) => {
                        eprintln!("error: {error}");
                    }
                }

                target_directory.context_menu.is_open_menu = false;

                sleep(Duration::from_millis(2000));

                execute!(
                    stdout,
                    MoveTo(0, 0),
                    LeaveAlternateScreen,
                    EnableMouseCapture, 
                    Hide
                ).unwrap();

                clear(stdout);
                target_directory.print_dir_content();
            },
            ContextMenuItems::Info => {
                execute!(
                    stdout,
                    MoveTo(0, 0),
                    EnterAlternateScreen,
                    DisableMouseCapture,
                    Show
                ).unwrap();

                let mut dir = fs::read_dir(&target_directory.path).unwrap();

                let dir_item = dir.nth(target_directory.selected).unwrap().unwrap();

                let mut selected_item = target_directory.path.clone();

                selected_item.push(dir_item.path());

                let info = metadata(&selected_item);

                match info {
                    Ok(info) => {
                        let item_type;

                        if info.is_dir() {
                            item_type = "dir";
                        } else if info.is_file() {
                            item_type = "file";
                        } else if info.is_symlink() {
                            item_type = "symbolic link";
                        } else {
                            item_type = "unknown type";
                        }

                        println!("type: {}", item_type);

                        println!("----------------------------------------------------------");

                        println!("path to the location: {}", selected_item.display());

                        println!("----------------------------------------------------------");

                        println!("size: {} bytes", info.len());

                        println!("----------------------------------------------------------");

                        if info.permissions().readonly() {
                            println!("permissions: readonly");
                        } else {
                            println!("permissions: read and write");
                        }

                        println!("----------------------------------------------------------");

                        if let Ok(time) = info.created() {
                            let time: DateTime<Local> = time.into();

                            println!("{} created: {:?}", item_type, time.to_rfc2822());
                        } else {
                            println!("it is impossible to determine the time of creation on this platform");
                        }

                        println!("----------------------------------------------------------");

                        if let Ok(time) = info.modified() {
                            let time: DateTime<Local> = time.into();

                            println!("{} modified: {:?}", item_type, time.to_rfc2822());
                        } else {
                            println!("Not supported on this platform");
                        }

                        println!("----------------------------------------------------------");

                        if let Ok(time) = info.accessed() {
                            let time: DateTime<Local> = time.into();

                            println!("{} accessed: {:?}", item_type, time.to_rfc2822());
                        } else {
                            println!("Not supported on this platform");
                        }
                    },
                    Err(error) => {
                        eprintln!("error: {error}");
                    }
                }

                println!("");
                println!("");
                println!("");
                print!("press enter to exit back: ");
                std::io::stdout().flush().expect("Помилка при очищенні буфера");

                let mut file_name = String::new();
                stdin().read_line(&mut file_name).expect("something went wrong when writing the name of file");

                target_directory.context_menu.is_open_menu = false;

                execute!(
                    stdout,
                    MoveTo(0, 0),
                    LeaveAlternateScreen,
                    EnableMouseCapture, 
                    Hide
                ).unwrap();

                clear(stdout);
                target_directory.print_dir_content();
            },
            ContextMenuItems::CopyPath => {
                let mut dir = fs::read_dir(&target_directory.path).unwrap();

                let dir_item = dir.nth(target_directory.selected).unwrap().unwrap();

                let mut path = target_directory.path.clone();

                path.push(dir_item.path());

                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();

                ctx.set_contents(String::from(path.to_str().unwrap())).unwrap();

                target_directory.context_menu.is_open_menu = false;

                clear(stdout);
                target_directory.print_dir_content();
            }
        }
    } else {
        target_directory.to_next_directory();
    
        clear(stdout);
        target_directory.print_dir_content();
    }

    Ok(())
}