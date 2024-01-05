use futures::{future::FutureExt, select, StreamExt};

use crossterm::execute;
use crossterm::event::EventStream;
use crossterm::event::EnableMouseCapture;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::MouseEventKind;
use crossterm::event::MouseButton;
use crossterm::cursor::Hide;
use crossterm::event::KeyEventKind;

use std::fs;
use std::{io::Result, io::stdout, io::Stdout};
use std::io::{self, Write};

use terminal_file_manager::target_directory::{TargetDirectory};
use terminal_file_manager::clear::{clear};
use terminal_file_manager::enter::{enter};

async fn keyboard_events(stdout: &mut Stdout, event_stream: &mut EventStream, target_directory: &mut TargetDirectory) -> Result<()> {
    match event_stream.next().await {
        Some(Ok(event)) => {
            if target_directory.fixing_double_click.captured_first_click && 
                target_directory.fixing_double_click.is_it_expired_time_between_clicks() 
            {
                target_directory.fixing_double_click.released_both_clicks();
            }

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
                                    let terminal_size = crossterm::terminal::size().unwrap();
                                    
                                    if target_directory.selected > (terminal_size.1 - 1) as usize {
                                        target_directory.change_selected_dir_or_file((target_directory.selected + mouse_event.row as usize) as usize);
                                    } else {
                                        target_directory.change_selected_dir_or_file(mouse_event.row as usize);
                                    }
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
                            mouse_event.row as usize <= fs::read_dir(&target_directory.path).unwrap().count()
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