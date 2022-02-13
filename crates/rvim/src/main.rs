// TODO: [✓] Streaming parsers for multi-key commands
//        * Store key command buffer in VimContext
//        * Try to parse on each input, consume on success
//        * Display buffer in UI
//        * Change cursor to underline when pending buffer matches "c", "d"
//
// TODO: [ ] Fix word motion edge cases
//        * Line 5 in leading whitespace, normal mode, b
//        * In trailing whitespace, normal mode, w
//        * Line 5 after first word, ^
//
// TODO: [✓] Generalize hardcoded commands
//        * XOut, open, etc
//        * nom parsers should be as compositional as possible
//        * Will need command composition functionality
//
// TODO: [✓] Implement substitute command
//
// TODO: [✓] Implement operators
//
// TODO: [ ] Implement more motions
//
// TODO: [ ] Implement more operators
//
// TODO: [ ] Implement visual mode
//
// TODO: [ ] Implement registers
//
// TODO: [ ] Implement undo/redo
//
// TODO: [ ] Implement internally mutable message buffer
//        * Need access from vim context methods as well as calling code
//        * Ex. 'Pattern not found' errors from vim context, command mode errors from calling code
//
// TODO: [>] Organize API, convert into library
// TODO: [✓] rename to vimbed
//
pub mod render;

use vimbed::{command::*, context::Context, nom::Err as NomErr};

use render::render;

use std::{
    borrow::Cow,
    error::Error,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::{stdout, Write};

fn main() -> Result<(), Box<dyn Error>> {
    let running = Rc::new(AtomicBool::new(true));

    let tabs = " ".repeat(4);

    let mut buffer_edit =
        "Testing One Two\nThree Four\n\tFive\n\n\t\tLorem Ipsum Dolor\n\t\tSit Amet"
            .replace('\t', &tabs);
    let mut buffer_command = Default::default();
    let mut buffer_search = Default::default();

    let mut ctx = Context::new(&mut buffer_edit, &mut buffer_command, &mut buffer_search)
        .with_command_callback({
            let mut stdout = stdout();
            let running = running.clone();
            move |command| {
                match command {
                    "write" | "w" => write!(stdout, "Write").unwrap(),
                    "edit" | "e" => write!(stdout, "Edit").unwrap(),
                    "quit" | "q" => running.store(false, Ordering::Relaxed),
                    _ => (),
                }
                stdout.flush().unwrap();
            }
        });

    let (mut width, mut height) = terminal::size()?;

    let mut so = stdout();
    so.queue(terminal::EnterAlternateScreen)?;
    so.queue(cursor::MoveTo(0, 0))?;
    so.flush()?;

    terminal::enable_raw_mode()?;

    let finalize = || -> Result<(), Box<dyn Error>> {
        terminal::disable_raw_mode()?;
        stdout().execute(terminal::LeaveAlternateScreen)?;
        Ok(())
    };

    std::panic::set_hook(Box::new(move |panic_info| {
        finalize().unwrap();
        println!(
            "Thread '{}' {}",
            std::thread::current()
                .name()
                .map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Owned(format!("{:?}", std::thread::current().id()))),
            panic_info
        );
    }));

    let mut input_buffer = String::default();

    render(&mut so, &ctx, &input_buffer, (width, height))?;

    loop {
        if !running.load(Ordering::Relaxed) {
            break;
        }

        if poll(Duration::default())? {
            match read()? {
                Event::Key(event) => match event {
                    crossterm::event::KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: crossterm::event::KeyModifiers::CONTROL,
                    } => break,
                    crossterm::event::KeyEvent { code, .. } => {
                        let input: Cow<'static, str> = match code {
                            KeyCode::Char(c) => c.to_string().into(),
                            KeyCode::Backspace => COMMAND_BACKSPACE.into(),
                            KeyCode::Enter => COMMAND_CARRIAGE_RETURN.into(),
                            KeyCode::F(n) => format!("<F{}>", n).into(),
                            KeyCode::Esc => COMMAND_ESCAPE.into(),
                            KeyCode::Delete => COMMAND_DELETE.into(),
                            KeyCode::Left => COMMAND_LEFT.into(),
                            KeyCode::Right => COMMAND_RIGHT.into(),
                            KeyCode::Up => COMMAND_UP.into(),
                            KeyCode::Down => COMMAND_DOWN.into(),
                            KeyCode::Home => COMMAND_HOME.into(),
                            KeyCode::End => COMMAND_END.into(),
                            _ => continue,
                        };

                        input_buffer += &input;

                        match ctx.input_str(&input_buffer) {
                            Ok(_) => {
                                input_buffer.clear();
                            }
                            Err(e) => match e {
                                NomErr::Incomplete(_) => (),              // Incomplete input, do nothing
                                NomErr::Error(_) => input_buffer.clear(), // Unhandled input, discard
                                NomErr::Failure(e) => panic!("{}", e), // Unrecoverable error, panic
                            },
                        };
                    }
                },
                Event::Resize(w, h) => {
                    width = w;
                    height = h;
                }
                _ => (),
            }

            render(&mut so, &ctx, &input_buffer, (width, height))?;
        }
    }

    finalize()?;

    Ok(())
}
