use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use systeroid_core::config::Config;
use systeroid_core::sysctl::controller::Sysctl;
use systeroid_core::sysctl::parameter::Parameter;
use systeroid_core::sysctl::section::Section;
use systeroid_tui::app::App;
use systeroid_tui::command::Command;
use systeroid_tui::error::Result;
use systeroid_tui::options::{Direction, ScrollArea};
use systeroid_tui::ui::render;
use tui::backend::{Backend, TestBackend};
use tui::buffer::Buffer;
use tui::Terminal;

fn assert_buffer(mut buffer: Buffer, backend: &TestBackend) -> Result<()> {
    assert_eq!(buffer.area, backend.size()?);
    for x in 0..buffer.area().width {
        for y in 0..buffer.area().height {
            buffer
                .get_mut(x, y)
                .set_style(backend.buffer().get(x, y).style());
        }
    }
    backend.assert_buffer(&buffer);
    Ok(())
}

#[test]
fn test_tui() -> Result<()> {
    let mut sysctl = Sysctl {
        parameters: vec![
            Parameter {
                name: String::from("user.name"),
                value: String::from("system"),
                description: None,
                section: Section::User,
                docs_path: PathBuf::new(),
                docs_title: String::new(),
            },
            Parameter {
                name: String::from("kernel.fictional.test_param"),
                value: String::from("0"),
                description: Some(String::from("This is a fictional parameter for testing")),
                section: Section::Kernel,
                docs_path: PathBuf::from("/etc/cosmos"),
                docs_title: String::from("Test Parameter"),
            },
            Parameter {
                name: String::from("vm.stat_interval"),
                value: String::from("1"),
                description: Some(String::from(
                    "The time interval between which vm statistics are updated",
                )),
                section: Section::Vm,
                docs_path: PathBuf::from("/usr/share/doc/linux/admin-guide/sysctl/vm.rst"),
                docs_title: String::from("stat_interval"),
            },
        ],
        config: Config::default(),
    };
    let mut app = App::new(&mut sysctl);
    app.section_list.state.select(None);

    let backend = TestBackend::new(40, 10);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| render(frame, &mut app))?;
    assert_buffer(
        Buffer::with_lines(vec![
            "╭Parameters────────────────────────────╮",
            "│user.name                   system    │",
            "│kernel.fictional.test_param 0         │",
            "│vm.stat_interval            1         │",
            "│                                      │",
            "│                                      │",
            "│                                      │",
            "│                                      │",
            "│                                  1/3 │",
            "╰──────────────────────────────────────╯",
        ]),
        terminal.backend(),
    )?;

    app.run_command(Command::Select)?;
    terminal.draw(|frame| render(frame, &mut app))?;
    assert_buffer(
        Buffer::with_lines(vec![
            "╭Parameters────────────────────────────╮",
            "│user.name                   system    │",
            "│kernel.fictional.test_param 0         │",
            "│vm.stat_interval            1         │",
            "│                                      │",
            "│                                  1/3 │",
            "╰──────────────────────────────────────╯",
            "╭──────────────────────────────────────╮",
            "│:set user.name system                 │",
            "╰──────────────────────────────────────╯",
        ]),
        terminal.backend(),
    )?;

    app.run_command(Command::ClearInput(false))?;
    app.run_command(Command::MoveCursor(2))?;
    app.run_command(Command::ClearInput(true))?;
    app.run_command(Command::ClearInput(true))?;
    "kill"
        .chars()
        .try_for_each(|c| app.run_command(Command::UpdateInput(c)))?;

    terminal.draw(|frame| render(frame, &mut app))?;
    assert_buffer(
        Buffer::with_lines(vec![
            "╭Parameters────────────────────────────╮",
            "│user.name                   system    │",
            "│kernel.fictional.test_param 0         │",
            "│vm.stat_interval            1         │",
            "│                                      │",
            "│                                  1/3 │",
            "╰──────────────────────────────────────╯",
            "╭──────────────────────────────────────╮",
            "│:set user.name syskill                │",
            "╰──────────────────────────────────────╯",
        ]),
        terminal.backend(),
    )?;

    app.run_command(Command::ProcessInput)?;
    terminal.draw(|frame| render(frame, &mut app))?;
    assert_buffer(
        Buffer::with_lines(vec![
            "╭Parameters────────────────────────────╮",
            "│user.name                   system    │",
            "│kernel.fictional.test_param 0         │",
            "│vm.stat_interval            1         │",
            "│                                      │",
            "│                                  1/3 │",
            "╰──────────────────────────────────────╯",
            "╭──────────────────────────────────────╮",
            "│MSG: sysctl error: `no such sysctl: us│",
            "╰──────────────────────────────────────╯",
        ]),
        terminal.backend(),
    )?;

    thread::sleep(Duration::from_millis(2000));
    app.tick();
    terminal.draw(|frame| render(frame, &mut app))?;
    assert_buffer(
        Buffer::with_lines(vec![
            "╭Parameters────────────────────────────╮",
            "│user.name                   system    │",
            "│kernel.fictional.test_param 0         │",
            "│vm.stat_interval            1         │",
            "│                                      │",
            "│                                      │",
            "│                                      │",
            "│                                      │",
            "│                                  1/3 │",
            "╰──────────────────────────────────────╯",
        ]),
        terminal.backend(),
    )?;

    app.run_command(Command::Search)?;
    app.run_command(Command::Exit)?;
    app.run_command(Command::Copy)?;
    terminal.draw(|frame| render(frame, &mut app))?;
    assert_buffer(
        Buffer::with_lines(vec![
            "╭Parameters────────────────────────────╮",
            "│user.name                   system    │",
            "│kernel.fictional.test_param 0         │",
            "│vm.sta╭───Copy to clipboard────╮      │",
            "│      │Parameter name          │      │",
            "│      │Parameter value         │      │",
            "│      ╰────────────────────────╯      │",
            "│                                      │",
            "│                                  1/3 │",
            "╰──────────────────────────────────────╯",
        ]),
        terminal.backend(),
    )?;

    app.run_command(Command::Scroll(ScrollArea::List, Direction::Down, 1))?;
    app.run_command(Command::Select)?;
    terminal.draw(|frame| render(frame, &mut app))?;
    assert_buffer(
        Buffer::with_lines(vec![
            "╭Parameters────────────────────────────╮",
            "│user.name                   system    │",
            "│kernel.fictional.test_param 0         │",
            "│vm.stat_interval            1         │",
            "│                                      │",
            "│                                  1/3 │",
            "╰──────────────────────────────────────╯",
            "╭──────────────────────────────────────╮",
            "│MSG: Clipboard support is not enabled │",
            "╰──────────────────────────────────────╯",
        ]),
        terminal.backend(),
    )?;

    app.input = Some(String::new());
    app.run_command(Command::Search)?;
    app.run_command(Command::UpdateInput('_'))?;
    terminal.draw(|frame| render(frame, &mut app))?;
    assert_buffer(
        Buffer::with_lines(vec![
            "╭Parameters────────╮╭──Documentation───╮",
            "│kernel.fictional.t││Test Parameter    │",
            "│vm.stat_interval =││==============    │",
            "│                  ││This is a         │",
            "│                  ││fictional         │",
            "│              1/2 ││parameter for     │",
            "╰──────────────────╯│testing           │",
            "╭──────────────────╮│-                 │",
            "│/_                ││Parameter:        │",
            "╰──────────────────╯╰──────────────────╯",
        ]),
        terminal.backend(),
    )?;

    app.run_command(Command::ProcessInput)?;
    app.run_command(Command::Scroll(ScrollArea::Documentation, Direction::Up, 1))?;
    app.run_command(Command::Scroll(
        ScrollArea::Documentation,
        Direction::Down,
        5,
    ))?;
    terminal.draw(|frame| render(frame, &mut app))?;
    assert_buffer(
        Buffer::with_lines(vec![
            "╭Parameters────────╮╭──Documentation───╮",
            "│kernel.fictional.t││This is a         │",
            "│vm.stat_interval =││fictional         │",
            "│                  ││parameter for     │",
            "│                  ││testing           │",
            "│                  ││-                 │",
            "│                  ││Parameter:        │",
            "│                  ││kernel.fictional.t│",
            "│              1/2 ││est_param         │",
            "╰──────────────────╯╰──────────────────╯",
        ]),
        terminal.backend(),
    )?;

    app.run_command(Command::Scroll(ScrollArea::List, Direction::Bottom, 1))?;
    app.run_command(Command::Scroll(ScrollArea::List, Direction::Top, 1))?;
    app.run_command(Command::Scroll(ScrollArea::List, Direction::Down, 1))?;
    app.run_command(Command::Refresh)?;
    terminal.draw(|frame| render(frame, &mut app))?;
    assert_buffer(
        Buffer::with_lines(vec![
            "╭Parameters────────╮╭──Documentation───╮",
            "│kernel.fictional.t││stat_interval     │",
            "│vm.stat_interval =││=============     │",
            "│                  ││The time interval │",
            "│                  ││between which vm  │",
            "│                  ││statistics are    │",
            "│                  ││updated           │",
            "│                  ││-                 │",
            "│              2/2 ││Parameter:        │",
            "╰──────────────────╯╰──────────────────╯",
        ]),
        terminal.backend(),
    )?;

    Ok(())
}
