use futures::{future::FutureExt, select, StreamExt};

use crossterm::cursor::RestorePosition;
use crossterm::execute;
use crossterm::event::EventStream;
use crossterm::style::ResetColor;
use crossterm::style::{Color::{Green, Red, Black, White, Magenta, Rgb}, Colors, SetColors};
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::cursor::MoveTo;
use crossterm::event::EnableMouseCapture;
use crossterm::event::DisableMouseCapture;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::MouseEventKind;
use crossterm::event::MouseButton;
use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::event::KeyEventKind;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::BeginSynchronizedUpdate;
use crossterm::terminal::EndSynchronizedUpdate;
use crossterm::terminal::SetSize;
use crossterm::style::SetStyle;
use crossterm::style::ContentStyle;
use crossterm::style::Attributes;
use crossterm::style::Attribute;

use chrono::DateTime;
use chrono::offset::Local;

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use std::{fs, env, error};
use std::{io::Result, io::stdout, io::Stdout};
use std::path::Path;
use std::io::{self, Write};
use std::path::PathBuf;
use std::string::ToString;
use std::io::stdin;
use std::fs::File;
use std::fs::create_dir;
use std::fs::remove_dir;
use std::fs::remove_file;
use std::fs::metadata;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;
use std::process::Command;
use std::env::consts::OS;

enum ContextMenuItems {
    DeleteItem,
    CreateDir,
    CreateFile,
    Copy,
    Paste,
    CutItem,
    Rename,
    Info,
    CopyPath
}

impl ToString for ContextMenuItems {
    fn to_string(&self) -> String {
        match self {
            ContextMenuItems::DeleteItem => {
                String::from("видалити")
            },
            ContextMenuItems::CreateDir => {
                String::from("створити папку")
            },
            ContextMenuItems::CreateFile => {
                String::from("створити файл")
            },
            ContextMenuItems::Copy => {
                String::from("скопіювати")
            },
            ContextMenuItems::Paste => {
                String::from("вставити")
            },
            ContextMenuItems::CutItem => {
                String::from("вирізати")
            },
            ContextMenuItems::Rename => {
                String::from("перейменувати")
            },
            ContextMenuItems::Info => {
                String::from("інформація про файл/папку")
            },
            ContextMenuItems::CopyPath => {
                String::from("скопіювати шлях до файлу/папки в буфер обміну")
            }
        }
    }
}

struct ContextMenu {
    menu: Vec<ContextMenuItems>,
    is_open_menu: bool,
    selected: usize,
    remember_way_to_selected_item: Option<PathBuf>,
    need_to_cut_elmenet: bool,
    start_context_menu_row: u16,
    start_context_menu_column: u16,
    largest_element_len: u16,
}

impl ContextMenu {
    fn new() -> Self {
        let menu = vec![
            ContextMenuItems::DeleteItem,
            ContextMenuItems::CreateDir,
            ContextMenuItems::CreateFile,
            ContextMenuItems::Copy,
            ContextMenuItems::Paste,
            ContextMenuItems::CutItem,
            ContextMenuItems::Rename,
            ContextMenuItems::Info,
            ContextMenuItems::CopyPath
        ];

        Self {
            menu,
            is_open_menu: false,
            selected: 0,
            remember_way_to_selected_item: None,
            need_to_cut_elmenet: false,
            start_context_menu_row: 0,
            start_context_menu_column: 0,
            largest_element_len: 0,
        }
    }

    fn draw_row_empty_spase(
        &self, 
        start_column_position: u16, 
        end_column_position: u16,
        row: u16
    ) {
        let mut column = start_column_position;

        while column <= end_column_position {
            execute!(
                stdout(), 
                MoveTo(
                    self.start_context_menu_column + column, 
                    self.start_context_menu_row + row
                )
            ).unwrap();

            print!(" ");

            column += 1;
        }
    }

    fn find_largest_menu_item_len(&mut self) {
        let mut largest_element: u16 = 0;

        self.menu.iter().for_each(|element| {
            let len = element.to_string().chars().count() as u16;

            if len > largest_element {
                largest_element = len;
            }
        });

        self.largest_element_len = largest_element;
    }

    fn print_context_menu(&mut self, column: u16, row: u16) {
        if !self.is_open_menu {
            self.start_context_menu_row = row;
            self.start_context_menu_column = column;

            self.find_largest_menu_item_len();
        }

        execute!(
            stdout(),
            SetColors(Colors::new(Black, White))
        ).unwrap();

        let mut row_counter = 0;

        self.draw_row_empty_spase(
            0, 
            self.largest_element_len + 1, 
            row_counter
        );

        row_counter += 1;

        for element in self.menu.iter().enumerate() {
            execute!(
                stdout(), 
                MoveTo(
                    self.start_context_menu_column, 
                    self.start_context_menu_row + row_counter
                )
            ).unwrap();

            if element.0 == self.selected {
                execute!(
                    stdout(),
                    SetColors(Colors::new(White, Magenta))
                ).unwrap();

                println!(" {}", element.1.to_string());

                self.draw_row_empty_spase(
                    (element.1.to_string().chars().count() + 1) as u16, 
                    self.largest_element_len + 1, 
                    row_counter
                );
            } else {
                execute!(
                    stdout(),
                    SetColors(Colors::new(Black, White))
                ).unwrap();

                println!(" {}", element.1.to_string());

                self.draw_row_empty_spase(
                    (element.1.to_string().chars().count() + 1) as u16, 
                    self.largest_element_len + 1, 
                    row_counter
                );
            }

            row_counter += 1;
        }

        execute!(
            stdout(),
            SetColors(Colors::new(Black, White))
        ).unwrap();

        self.draw_row_empty_spase(
            0, 
            self.largest_element_len + 1, 
            row_counter
        );

        execute!(
            stdout(),
            ResetColor
        ).unwrap();

        self.is_open_menu = true;
    }
}

struct FixingDoubleClick {
    allowable_time_between_clicks: Duration,
    captured_first_click: bool,
    captured_second_click: bool,
    result_time_between_clicks: Option<Instant>
}

impl FixingDoubleClick {
    fn new() -> Self {
        Self {
            allowable_time_between_clicks: Duration::from_millis(500),
            captured_first_click: false,
            captured_second_click: false,
            result_time_between_clicks: None
        }
    }

    fn first_click(&mut self) {
        self.result_time_between_clicks = Some(Instant::now());
        self.captured_first_click = true;
    }

    fn second_click(&mut self) {
        self.captured_second_click = true;
    }

    fn check_it_is_double_click(&mut self) -> bool {
        if self.captured_first_click && self.captured_second_click {
            match self.result_time_between_clicks {
                Some(result_time) => {
                    self.captured_second_click = false;
                    self.captured_first_click = false;

                    if result_time.elapsed() <= self.allowable_time_between_clicks {
                        return true;
                    }
                },
                None => {
                    return false;
                }
            }
        }

        return false;
    }
}

struct TargetDirectory {
    path: PathBuf,
    selected: usize,
    context_menu: ContextMenu,
    fixing_double_click: FixingDoubleClick
}

impl TargetDirectory {
    fn new() -> Self {
        let path = env::current_dir().expect("Помилка при отриманні каталогу");

        Self {
            path,
            selected: 0,
            context_menu: ContextMenu::new(),
            fixing_double_click: FixingDoubleClick::new()
        }
    }

    fn to_previous_directory(&mut self) {
        self.path.pop();

        self.change_selected_dir_or_file(0);
        self.fixing_double_click.captured_second_click = false;
        self.fixing_double_click.captured_first_click = false;
    }

    fn to_next_directory(&mut self) {
        let mut dir = fs::read_dir(&self.path).unwrap();

        let dir_item = dir.nth(self.selected).unwrap().unwrap();

        if self.path.join(dir_item.path()).is_dir() {
            self.path.push(dir_item.path());

            self.change_selected_dir_or_file(0);

            self.fixing_double_click.captured_second_click = false;
            self.fixing_double_click.captured_first_click = false;
        } else {
            self.open_file();

            self.fixing_double_click.captured_second_click = false;
            self.fixing_double_click.captured_first_click = false;
        }
    }

    fn open_file(&self) {
        let mut dir = fs::read_dir(&self.path).unwrap();

        let dir_item = dir.nth(self.selected).unwrap().unwrap();

        match OS {
            "windows" => {
                let open_file = Command::new("cmd")
                    .args(&[
                        "/C", 
                        "explorer", 
                        self.path.join(dir_item.path()).to_str().unwrap()
                    ])
                    .status()
                    .expect(format!("could not be opened {}", &dir_item.path().to_str().unwrap()));
            },
            "linux" => {
                let status = Command::new("xdg-open")
                    .arg(self.path.join(&dir_item.path()).to_str().unwrap())
                    .status()
                    .expect(format!("could not be opened {}", &dir_item.path().to_str().unwrap()));
            },
            _ => {
                println!("операційна система не підтримується");
            }
        }

        execute!(
            stdout(),
            MoveTo(0, 0),
            EnableMouseCapture, 
            Hide
        ).unwrap();
    }

    fn change_selected_dir_or_file(&mut self, selected: usize) {
        self.selected = selected;
    }

    fn print_dir_content(&self) {
        let dir_content = fs::read_dir(&self.path);

        match dir_content {
            Ok(content) => {
                for entry in content.enumerate() { // todo - зробити більш гарний "ui", щось накшталт того як зробив контекстне меню (також додати якихось бордерів чи щось таке)
                    if let (index, Ok(entry)) = entry {
                        if index == self.selected {
                            execute!(
                                stdout(),
                                SetColors(Colors::new(Black, Green))
                            ).unwrap();
        
                            println!("{}", entry.file_name().to_str().unwrap());
        
                            execute!(
                                stdout(),
                                ResetColor
                            ).unwrap();
                        } else {
                            println!("{}", entry.file_name().to_str().unwrap());
                        }
                    }
                }
            },
            Err(error) => {
                execute!(
                    stdout(),
                    SetColors(Colors::new(Black, Red))
                ).unwrap();

                eprintln!("{}", error);

                execute!(
                    stdout(),
                    ResetColor
                ).unwrap();
            }
        }
    }
}

fn clear(stdout: &mut Stdout) {
    execute!(
        stdout,
        Clear(ClearType::All), // todo - виправити використання двох типів очищення одночасно (якщо використовувати один з них нормального очищення на Windows не відбувається)
        Clear(ClearType::Purge),
        RestorePosition,
    ).unwrap();

}

fn enter(stdout: &mut Stdout, target_directory: &mut TargetDirectory) -> Result<()> {
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

                } else {
                    print!("ви дійсно хочете видалити файл {}? Введіть 'yes' або 'no'.", &dir_item.file_name().to_str().unwrap());
                    std::io::stdout().flush().expect("Помилка при очищенні буфера");

                    let mut entered = String::new();
                    stdin().read_line(&mut entered).expect("something went wrong when writing the name of directory");
                    
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

async fn keyboard_events(stdout: &mut Stdout, event_stream: &mut EventStream, target_directory: &mut TargetDirectory) -> Result<()> {
    match event_stream.next().await {
        Some(Ok(event)) => {
            match event {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Release {
                        if key_event.code == KeyCode::Up {
                            if target_directory.context_menu.is_open_menu {
                                if target_directory.context_menu.selected.checked_sub(1) != None {
                                    target_directory.context_menu.selected -= 1;
    
                                    clear(stdout);
                                    target_directory.print_dir_content();
                                    target_directory.context_menu.print_context_menu(
                                        target_directory.context_menu.start_context_menu_column, 
                                        target_directory.context_menu.start_context_menu_row
                                    );
                                }
                            } else {
                                if target_directory.selected.checked_sub(1) != None {
                                    target_directory.change_selected_dir_or_file(target_directory.selected - 1);
                            
                                    clear(stdout);
                                    target_directory.print_dir_content();
                                }
                            }
                        } else if key_event.code == KeyCode::Down {
                            if target_directory.context_menu.is_open_menu {
                                if target_directory.context_menu.selected + 1 < target_directory.context_menu.menu.len() {
                                    target_directory.context_menu.selected += 1;
                                    
                                    clear(stdout);
                                    target_directory.print_dir_content();
                                    target_directory.context_menu.print_context_menu(
                                        target_directory.context_menu.start_context_menu_column, 
                                        target_directory.context_menu.start_context_menu_row
                                    );
                                }
                            } else {
                                if target_directory.selected + 1 < fs::read_dir(&target_directory.path).unwrap().count() {
                                    target_directory.change_selected_dir_or_file(target_directory.selected + 1);
                                    
                                    clear(stdout);
                                    target_directory.print_dir_content();
                                }
                            }
                        } else if key_event.code == KeyCode::Left {
                            target_directory.to_previous_directory();
                            
                            clear(stdout);
                            target_directory.print_dir_content();
                        } else if key_event.code == KeyCode::Right {
                            target_directory.to_next_directory();
                            
                            clear(stdout);
                            target_directory.print_dir_content();
                        } else if key_event.code == KeyCode::Enter {
                            enter(stdout, target_directory).unwrap();
                        } else if key_event.code == KeyCode::Esc {
                            target_directory.to_previous_directory();
                            
                            clear(stdout);
                            target_directory.print_dir_content();
                        } else {
                            
                        }
                    }
                },
                Event::Mouse(mouse_event) => {
                    if mouse_event.kind == MouseEventKind::Up(MouseButton::Left) {
                        if !target_directory.fixing_double_click.captured_first_click {
                            target_directory.fixing_double_click.first_click();
                        } else {
                            target_directory.fixing_double_click.second_click();
                        }

                        if target_directory.fixing_double_click.check_it_is_double_click() {
                            enter(stdout, target_directory).unwrap();
                        } else {
                            if target_directory.context_menu.is_open_menu {
                                if mouse_event.row < target_directory.context_menu.start_context_menu_row ||
                                    mouse_event.row > (
                                        target_directory.context_menu.start_context_menu_row + 
                                        target_directory.context_menu.menu.len() as u16 + 1
                                    ) ||
                                    mouse_event.column < target_directory.context_menu.start_context_menu_column ||
                                    mouse_event.column > (
                                        target_directory.context_menu.start_context_menu_column + 
                                        target_directory.context_menu.largest_element_len + 1
                                    )
                                {
                                    target_directory.context_menu.is_open_menu = false;
                                } else {
                                    if (mouse_event.row.saturating_sub(target_directory.context_menu.start_context_menu_row)) > 0 &&
                                        (mouse_event.row.saturating_sub(target_directory.context_menu.start_context_menu_row.saturating_add(1))) < 
                                        target_directory.context_menu.menu.len() as u16 
                                    {
                                        target_directory.context_menu.selected = (mouse_event.row.saturating_sub(target_directory.context_menu.start_context_menu_row.saturating_add(1))) as usize;
                                    }
                                    
                                }
                            } else {
                                if mouse_event.row as usize <= fs::read_dir(&target_directory.path).unwrap().count() {
                                    target_directory.change_selected_dir_or_file(mouse_event.row as usize);
                                }
                            }
                        }

                        clear(stdout);
                        target_directory.print_dir_content();

                        if target_directory.context_menu.is_open_menu {
                            target_directory.context_menu.print_context_menu(mouse_event.column, mouse_event.row);
                        }
                    } else if mouse_event.kind == MouseEventKind::Up(MouseButton::Right) {
                        if !target_directory.context_menu.is_open_menu && 
                            mouse_event.row as usize <= fs::read_dir(&target_directory.path).unwrap().count() &&
                            mouse_event.row == target_directory.selected as u16
                        {
                            target_directory.context_menu.print_context_menu(mouse_event.column, mouse_event.row);
                        }
                        
                    } else if mouse_event.kind == MouseEventKind::ScrollUp {
                        if target_directory.context_menu.is_open_menu {
                            if target_directory.context_menu.selected.checked_sub(1) != None {
                                target_directory.context_menu.selected -= 1;

                                clear(stdout);
                                target_directory.print_dir_content();
                                target_directory.context_menu.print_context_menu(
                                    target_directory.context_menu.start_context_menu_column, 
                                    target_directory.context_menu.start_context_menu_row
                                );
                            }
                        } else {
                            if target_directory.selected.checked_sub(1) != None {
                                target_directory.change_selected_dir_or_file(target_directory.selected - 1);

                                clear(stdout);
                                target_directory.print_dir_content();
                            }
                        }
                    } else if mouse_event.kind == MouseEventKind::ScrollDown {
                        if target_directory.context_menu.is_open_menu {
                            if target_directory.context_menu.selected + 1 < target_directory.context_menu.menu.len() {
                                target_directory.context_menu.selected += 1;
                                
                                clear(stdout);
                                target_directory.print_dir_content();
                                target_directory.context_menu.print_context_menu(
                                    target_directory.context_menu.start_context_menu_column, 
                                    target_directory.context_menu.start_context_menu_row
                                );
                            }
                        } else {
                            if target_directory.selected + 1 < fs::read_dir(&target_directory.path).unwrap().count() {
                                target_directory.change_selected_dir_or_file(target_directory.selected + 1);

                                clear(stdout);
                                target_directory.print_dir_content();
                            }
                        }
                    } else {

                        
                    }
                },
                _ => {  }
            }
        },
        Some(Err(error)) => {
            eprintln!("error: {}", error);
        }
        None => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut stdout = stdout();

    let mut event_stream = EventStream::new();

    let mut target_directory = TargetDirectory::new();

    execute!(
        stdout,
        //SetSize(400, 400),
        /*SetStyle(
            ContentStyle {
                foreground_color: Some(White),
                background_color: Some(Rgb {
                    r: 37,
                    g: 37,
                    b: 37,
                }),
                underline_color: None,
                attributes: Attribute::Bold.into(),
            }
        ),*/
        EnableMouseCapture, 
        Hide
    ).unwrap();

    clear(&mut stdout);
    target_directory.print_dir_content();

    loop {
        let _ = keyboard_events(&mut stdout, &mut event_stream, &mut target_directory).await;
    }
}