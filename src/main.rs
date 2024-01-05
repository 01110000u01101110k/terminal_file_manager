use futures::{future::FutureExt, select, StreamExt};

use crossterm::execute;
use crossterm::event::EventStream;
use crossterm::event::EnableMouseCapture;
use crossterm::cursor::Hide;

use std::{io::Result, io::stdout, io::Stdout};

use terminal_file_manager::target_directory::{TargetDirectory};
use terminal_file_manager::clear::{clear};
use terminal_file_manager::keyboard_events::{keyboard_events};

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