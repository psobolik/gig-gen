mod args;
mod gitignore_api;
mod template;

use crate::template::Templates;
use args::{Args, Commands, FilterArgs};
use cursive::{
    align::HAlign,
    event::{Event, EventResult, Key},
    menu,
    style::{
        BaseColor, BorderStyle, Color, ColorStyle, Effect, Palette, PaletteColor, PaletteStyle,
        Style,
    },
    theme::Theme,
    traits::*,
    utils::markup::StyledString,
    views::{Dialog, DummyView, LinearLayout, OnEventView, SelectView, TextView},
    Cursive,
};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

const AVAILABLE_VIEW_NAME: &str = "available";
const SELECTED_VIEW_NAME: &str = "selected";
const FILTER_VIEW_NAME: &str = "filter";
const OUTPUT_FILE_NAME: &str = ".gitignore";

type CbSink = crossbeam_channel::Sender<Box<dyn FnOnce(&mut Cursive) + Send>>;

enum SaveOption {
    Create,
    Overwrite,
    Append,
}

struct UserData {
    templates: Templates,
    filter: String,
    new_filter: bool,
    cb_sink: CbSink,
    final_message: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let args: Args = clap::Parser::parse();
    match args.command.unwrap_or(Commands::Interactive) {
        Commands::List(args) => list_templates(args),
        Commands::Generate(args) => generate_gitignore(args.templates),
        Commands::Interactive => {
            interactive();
            Ok(())
        }
    }
}
fn list_templates(args: FilterArgs) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut templates = gitignore_api::get_template_names()?;
    if let Some(filter) = args.filter {
        let filter = regex::escape(filter.as_str());
        let re = regex::Regex::new(filter.as_str())?;
        templates = templates
            .iter()
            .filter_map(|t| {
                if re.is_match(t) {
                    Some(t.to_string())
                } else {
                    None
                }
            })
            .collect();
        if templates.is_empty() {
            print_message(format!(r#"No templates match "{}""#, filter).as_str());
            // println!(r#"No templates match "{}""#, filter);
        }
    }
    for template in templates {
        println!("{}", template);
    }
    Ok(())
}
fn generate_gitignore(
    template_names: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    match gitignore_api::get_template(&template_names) {
        Ok(result) => {
            println!("{}", result);
            Ok(())
        }
        Err(error) => {
            let message = format!(
                r#"Problem getting .gitignore for "{}": "#,
                template_names.join(" ")
            );
            print_message(message.as_str());
            Err(error.into())
        }
    }
}
fn interactive() {
    fn load_templates() -> Templates {
        let mut templates = Templates::new();
        if let Ok(template_names) = gitignore_api::get_template_names() {
            templates.set_list(template_names);
        };
        templates
        // TODO: Handle error?
    }
    let mut siv = cursive::default();
    siv.add_global_callback(Event::CtrlChar('q'), |siv| siv.quit());
    siv.add_global_callback(Event::CtrlChar('s'), save);
    siv.add_global_callback(Event::Key(Key::F1), help);
    let user_data = UserData {
        templates: load_templates(),
        filter: String::default(),
        new_filter: false,
        cb_sink: siv.cb_sink().clone(),
        final_message: None,
    };
    siv.set_user_data(user_data);
    siv.set_theme(theme());

    siv.menubar()
        .add_subtree(
            "File",
            menu::Tree::new().with(|tree| {
                tree.add_leaf("Save ^S", save);
                tree.add_leaf("Quit ^Q", Cursive::quit);
            }),
        )
        .add_subtree(
            "Help",
            menu::Tree::new().with(|tree| {
                tree.add_leaf("Help F1", help);
                tree.add_leaf("About", about);
            }),
        );
    siv.set_autohide_menu(false);

    siv.add_fullscreen_layer(event_view(main_layer()));
    refresh(&mut siv);
    siv.run();
    siv.with_user_data(|user_data: &mut UserData| {
        if let Some(final_message) = &user_data.final_message {
            print_message(final_message);
        }
    });
}
fn print_message(message: &str) {
    eprintln!("[{}] \x1b[93m{}\x1b[0m", env!["CARGO_PKG_NAME"], message);
}
fn theme() -> Theme {
    Theme {
        shadow: false,
        borders: BorderStyle::Simple,
        palette: Palette::retro().with(|palette| {
            palette[PaletteColor::Background] = Color::TerminalDefault;
            palette[PaletteColor::View] = BaseColor::Black.dark();
            palette[PaletteColor::Primary] = BaseColor::White.light();
            palette[PaletteColor::Secondary] = BaseColor::Blue.light();
            palette[PaletteColor::Tertiary] = BaseColor::Yellow.light();
            palette[PaletteColor::Highlight] = BaseColor::Blue.dark();
            palette[PaletteColor::HighlightText] = BaseColor::White.light();
            palette[PaletteStyle::TitlePrimary] =
                Style::from(BaseColor::Yellow.light()).combine(Effect::Bold);
            palette[PaletteStyle::HighlightInactive] = Style::from(ColorStyle::new(
                BaseColor::White.light(),
                BaseColor::Black.light(),
            ));
        }),
    }
}
fn event_view(content: impl View) -> impl View {
    fn clear_filter(siv: &mut Cursive) {
        if let Some(user_data) = siv.user_data::<UserData>() {
            user_data.filter = String::default();
            user_data.new_filter = true;
        }
        refresh(siv);
    }
    fn backspace(siv: &mut Cursive) {
        if let Some(user_data) = siv.user_data::<UserData>() {
            if !user_data.filter.is_empty() {
                user_data.filter = user_data.filter[..user_data.filter.len() - 1].to_string();
                user_data.new_filter = true;
            }
        }
        refresh(siv);
    }
    fn handle_char(siv: &mut Cursive, c: char) {
        if let Some(user_data) = siv.user_data::<UserData>() {
            user_data.filter += c.to_string().as_str();
            user_data.new_filter = true;
        }
        refresh(siv);
    }
    OnEventView::new(content)
        .on_event(Event::Key(Key::Esc), clear_filter)
        .on_event(Event::Key(Key::Backspace), backspace)
        .on_event(Event::Char('a'), |s| handle_char(s, 'a'))
        .on_event(Event::Char('b'), |s| handle_char(s, 'b'))
        .on_event(Event::Char('c'), |s| handle_char(s, 'c'))
        .on_event(Event::Char('d'), |s| handle_char(s, 'd'))
        .on_event(Event::Char('e'), |s| handle_char(s, 'e'))
        .on_event(Event::Char('f'), |s| handle_char(s, 'f'))
        .on_event(Event::Char('g'), |s| handle_char(s, 'g'))
        .on_event(Event::Char('h'), |s| handle_char(s, 'h'))
        .on_event(Event::Char('i'), |s| handle_char(s, 'i'))
        .on_event(Event::Char('j'), |s| handle_char(s, 'j'))
        .on_event(Event::Char('k'), |s| handle_char(s, 'k'))
        .on_event(Event::Char('l'), |s| handle_char(s, 'l'))
        .on_event(Event::Char('m'), |s| handle_char(s, 'm'))
        .on_event(Event::Char('n'), |s| handle_char(s, 'n'))
        .on_event(Event::Char('o'), |s| handle_char(s, 'o'))
        .on_event(Event::Char('p'), |s| handle_char(s, 'p'))
        .on_event(Event::Char('q'), |s| handle_char(s, 'q'))
        .on_event(Event::Char('r'), |s| handle_char(s, 'r'))
        .on_event(Event::Char('s'), |s| handle_char(s, 's'))
        .on_event(Event::Char('t'), |s| handle_char(s, 't'))
        .on_event(Event::Char('u'), |s| handle_char(s, 'u'))
        .on_event(Event::Char('v'), |s| handle_char(s, 'v'))
        .on_event(Event::Char('w'), |s| handle_char(s, 'w'))
        .on_event(Event::Char('x'), |s| handle_char(s, 'x'))
        .on_event(Event::Char('y'), |s| handle_char(s, 'y'))
        .on_event(Event::Char('z'), |s| handle_char(s, 'z'))
        .on_event(Event::Char('A'), |s| handle_char(s, 'A'))
        .on_event(Event::Char('B'), |s| handle_char(s, 'B'))
        .on_event(Event::Char('C'), |s| handle_char(s, 'C'))
        .on_event(Event::Char('D'), |s| handle_char(s, 'D'))
        .on_event(Event::Char('E'), |s| handle_char(s, 'E'))
        .on_event(Event::Char('F'), |s| handle_char(s, 'F'))
        .on_event(Event::Char('G'), |s| handle_char(s, 'G'))
        .on_event(Event::Char('H'), |s| handle_char(s, 'H'))
        .on_event(Event::Char('I'), |s| handle_char(s, 'I'))
        .on_event(Event::Char('J'), |s| handle_char(s, 'J'))
        .on_event(Event::Char('K'), |s| handle_char(s, 'K'))
        .on_event(Event::Char('L'), |s| handle_char(s, 'L'))
        .on_event(Event::Char('M'), |s| handle_char(s, 'M'))
        .on_event(Event::Char('N'), |s| handle_char(s, 'N'))
        .on_event(Event::Char('O'), |s| handle_char(s, 'O'))
        .on_event(Event::Char('P'), |s| handle_char(s, 'P'))
        .on_event(Event::Char('Q'), |s| handle_char(s, 'Q'))
        .on_event(Event::Char('R'), |s| handle_char(s, 'R'))
        .on_event(Event::Char('S'), |s| handle_char(s, 'S'))
        .on_event(Event::Char('T'), |s| handle_char(s, 'T'))
        .on_event(Event::Char('U'), |s| handle_char(s, 'U'))
        .on_event(Event::Char('V'), |s| handle_char(s, 'V'))
        .on_event(Event::Char('W'), |s| handle_char(s, 'W'))
        .on_event(Event::Char('X'), |s| handle_char(s, 'X'))
        .on_event(Event::Char('Y'), |s| handle_char(s, 'Y'))
        .on_event(Event::Char('Z'), |s| handle_char(s, 'Z'))
        .on_event(Event::Char('0'), |s| handle_char(s, '0'))
        .on_event(Event::Char('1'), |s| handle_char(s, '1'))
        .on_event(Event::Char('2'), |s| handle_char(s, '2'))
        .on_event(Event::Char('3'), |s| handle_char(s, '3'))
        .on_event(Event::Char('4'), |s| handle_char(s, '4'))
        .on_event(Event::Char('5'), |s| handle_char(s, '5'))
        .on_event(Event::Char('6'), |s| handle_char(s, '6'))
        .on_event(Event::Char('7'), |s| handle_char(s, '7'))
        .on_event(Event::Char('8'), |s| handle_char(s, '8'))
        .on_event(Event::Char('9'), |s| handle_char(s, '9'))
        .on_event(Event::Char(' '), |s| handle_char(s, ' '))
        .on_event(Event::Char('!'), |s| handle_char(s, '!'))
        .on_event(Event::Char('"'), |s| handle_char(s, '"'))
        .on_event(Event::Char('#'), |s| handle_char(s, '#'))
        .on_event(Event::Char('$'), |s| handle_char(s, '$'))
        .on_event(Event::Char('%'), |s| handle_char(s, '%'))
        .on_event(Event::Char('\''), |s| handle_char(s, '\''))
        .on_event(Event::Char('('), |s| handle_char(s, '('))
        .on_event(Event::Char(')'), |s| handle_char(s, ')'))
        .on_event(Event::Char('*'), |s| handle_char(s, '*'))
        .on_event(Event::Char('+'), |s| handle_char(s, '+'))
        .on_event(Event::Char(','), |s| handle_char(s, ','))
        .on_event(Event::Char('-'), |s| handle_char(s, '-'))
        .on_event(Event::Char('.'), |s| handle_char(s, '.'))
        .on_event(Event::Char('/'), |s| handle_char(s, '/'))
        .on_event(Event::Char(':'), |s| handle_char(s, ':'))
        .on_event(Event::Char(';'), |s| handle_char(s, ';'))
        .on_event(Event::Char('<'), |s| handle_char(s, '<'))
        .on_event(Event::Char('='), |s| handle_char(s, '='))
        .on_event(Event::Char('>'), |s| handle_char(s, '>'))
        .on_event(Event::Char('?'), |s| handle_char(s, '?'))
        .on_event(Event::Char('\''), |s| handle_char(s, '\''))
        .on_event(Event::Char('['), |s| handle_char(s, '['))
        .on_event(Event::Char('\\'), |s| handle_char(s, '\\'))
        .on_event(Event::Char(']'), |s| handle_char(s, ']'))
        .on_event(Event::Char('^'), |s| handle_char(s, '^'))
        .on_event(Event::Char('_'), |s| handle_char(s, '_'))
        .on_event(Event::Char('`'), |s| handle_char(s, '`'))
        .on_event(Event::Char('{'), |s| handle_char(s, '{'))
        .on_event(Event::Char('|'), |s| handle_char(s, '|'))
        .on_event(Event::Char('}'), |s| handle_char(s, '}'))
        .on_event(Event::Char('~'), |s| handle_char(s, '~'))
}
fn main_layer() -> impl View {
    fn make_label(text: &str) -> impl View {
        TextView::new(StyledString::styled(text, BaseColor::Yellow.dark())).h_align(HAlign::Center)
    }
    fn make_layout(label: &str, name: &str, on_submit: fn(&mut Cursive, &str)) -> impl View {
        fn make_select_view(name: &str, on_submit: fn(&mut Cursive, &str)) -> impl View {
            SelectView::<String>::new()
                .on_submit(on_submit)
                .with_name(name)
                .scrollable()
                .wrap_with(OnEventView::new)
                .on_pre_event_inner(Event::CtrlChar('n'), |view, _event| {
                    view.on_event(Event::Key(Key::Down));
                    Some(EventResult::Consumed(None))
                })
                .on_pre_event_inner(Event::CtrlChar('p'), |view, _event| {
                    view.on_event(Event::Key(Key::Up));
                    Some(EventResult::Consumed(None))
                })
        }
        LinearLayout::vertical()
            .child(make_label(label))
            .child(make_select_view(name, on_submit))
            .min_width(29)
            .full_width()
            .full_height()
    }

    let lists_layout = LinearLayout::horizontal()
        .child(make_layout(
            " Available templates ",
            AVAILABLE_VIEW_NAME,
            select_item,
        ))
        .child(DummyView::new().fixed_width(4))
        .child(make_layout(
            " Selected templates ",
            SELECTED_VIEW_NAME,
            unselect_item,
        ));

    let filter_layout = LinearLayout::horizontal()
        .child(make_label("Filter:"))
        .child(TextView::new(String::default()).with_name(FILTER_VIEW_NAME));

    LinearLayout::vertical()
        .child(lists_layout)
        .child(DummyView::new())
        .child(filter_layout)
}
fn save(siv: &mut Cursive) {
    siv.with_user_data(|user_data: &mut UserData| {
        if user_data.templates.any_selected() {
            let output_file = Path::new(OUTPUT_FILE_NAME);
            if output_file.exists() {
                user_data
                    .cb_sink
                    .send(Box::new(get_overwrite_choice))
                    .expect("get overwrite choice failed");
            } else {
                user_data
                    .cb_sink
                    .send(Box::new(create))
                    .expect("create failed");
            }
        } else {
            user_data
                .cb_sink
                .send(Box::new(nothing_to_save_warning))
                .expect("save warning failed");
        }
    });
}
fn get_overwrite_choice(siv: &mut Cursive) {
    siv.add_layer(
        Dialog::text("A .gitignore file already\rexists in the current directory.")
            .h_align(HAlign::Center)
            .button("Overwrite", |s| {
                s.pop_layer();
                s.user_data::<UserData>()
                    .unwrap()
                    .cb_sink
                    .clone()
                    .send(Box::new(overwrite))
                    .expect("overwrite choice failed");
            })
            .button("Append", |s| {
                s.pop_layer();
                s.user_data::<UserData>()
                    .unwrap()
                    .cb_sink
                    .clone()
                    .send(Box::new(append))
                    .expect("append choice failed");
            })
            .button("Cancel", |s| {
                s.pop_layer();
            }),
    );
}
fn save_gitignore(siv: &mut Cursive, save_option: SaveOption) -> bool {
    fn get_gitignore(siv: &mut Cursive) -> Option<Result<String, minreq::Error>> {
        siv.with_user_data(|user_data: &mut UserData| {
            let selected_templates = user_data.templates.selected_template_names();
            gitignore_api::get_template(&selected_templates)
        })
    }
    if let Some(gitignore) = get_gitignore(siv) {
        let mut open_options = OpenOptions::new();
        open_options.write(true);
        match gitignore {
            Ok(gitignore) => {
                match save_option {
                    SaveOption::Create => {
                        open_options.create_new(true);
                    }
                    SaveOption::Overwrite => {
                        open_options.truncate(true);
                    }
                    SaveOption::Append => {
                        open_options.append(true);
                    }
                }
                match open_options.open(OUTPUT_FILE_NAME) {
                    Ok(mut file) => {
                        if let Err(error) = file.write(gitignore.as_bytes()) {
                            let message = format!("Error writing .gitignore file. [{}]", error);
                            siv.add_layer(Dialog::info(message).h_align(HAlign::Center));
                            false
                        } else {
                            true
                        }
                    }
                    Err(error) => {
                        let message = format!("Error opening .gitignore file. [{}]", error);
                        siv.add_layer(Dialog::info(message).h_align(HAlign::Center));
                        false
                    }
                }
            }
            Err(error) => {
                let message = format!("Error fetching .gitignore data. [{}]", error);
                siv.add_layer(Dialog::info(message).h_align(HAlign::Center));
                false
            }
        }
    } else {
        panic!("No user data?");
    }
}
fn overwrite(siv: &mut Cursive) {
    if save_gitignore(siv, SaveOption::Overwrite) {
        siv.with_user_data(|user_data: &mut UserData| {
            user_data.final_message =
                Some("Replaced contents of existing .gitignore file.".to_string());
        });
        siv.quit();
    }
}
fn append(siv: &mut Cursive) {
    if save_gitignore(siv, SaveOption::Append) {
        siv.with_user_data(|user_data: &mut UserData| {
            user_data.final_message =
                Some("Appended templates to existing .gitignore file.".to_string());
        });
        siv.quit();
    }
}
fn create(siv: &mut Cursive) {
    if save_gitignore(siv, SaveOption::Create) {
        siv.with_user_data(|user_data: &mut UserData| {
            user_data.final_message = Some("Created new .gitignore file.".to_string());
        });
        siv.quit();
    }
}
fn nothing_to_save_warning(siv: &mut Cursive) {
    siv.add_layer(
        Dialog::info("Select one or more templates and try again.").h_align(HAlign::Center),
    );
}
fn select_item(siv: &mut Cursive, selection: &str) {
    siv.with_user_data(|user_data: &mut UserData| {
        user_data.templates.select_template(selection);
    });
    refresh(siv);
}
fn unselect_item(siv: &mut Cursive, selection: &str) {
    siv.with_user_data(|user_data: &mut UserData| {
        user_data.templates.unselect_template(selection);
    });
    refresh(siv);
}
fn refresh(siv: &mut Cursive) {
    let mut available_view = siv
        .find_name::<SelectView<String>>(AVAILABLE_VIEW_NAME)
        .unwrap();
    let available_index = available_view.selected_id().unwrap_or_default();

    let mut selected_view = siv.find_name::<SelectView>(SELECTED_VIEW_NAME).unwrap();

    let mut filter_view = siv.find_name::<TextView>(FILTER_VIEW_NAME).unwrap();

    siv.with_user_data(|user_data: &mut UserData| {
        // Display the filter
        filter_view.set_content(format!(" {}", user_data.filter));

        // Display the possibly filtered list of available templates
        available_view.clear();
        user_data
            .templates
            .unselected_templates()
            .iter()
            .filter_map(|template| {
                if !user_data.filter.is_empty()
                    && !template.name().starts_with(user_data.filter.as_str())
                {
                    None
                } else {
                    Some(template.name())
                }
            })
            .for_each(|template_name| available_view.add_item_str(template_name));

        // Set the selected item unless the list has just been filtered
        if !user_data.new_filter {
            available_view.set_selection(available_index);
            user_data.new_filter = false;
        }

        // Display the list of selected templates
        selected_view.clear();
        user_data
            .templates
            .selected_templates()
            .iter()
            .for_each(|option| selected_view.add_item_str(option.name()));
    });
}
fn help(siv: &mut Cursive) {
    let message = "Use this app to create a .gitignore file for one or more operating systems, programming languages or IDEs, using templates from https://www.toptal.com/developers/gitignore/

Select the templates to include in the file.
- Use the up and down arrows to highlight a template.
- Press Enter to select the highlighted template.
- Type the start of the template's name to filter the list.

Press Ctrl+S to write the .gitignore file to disk.
- The .gitignore file will be written to the current directory.
- If the .gitignore file already exists, you will be given the option of replacing it or appending to it.

Press Ctrl+Q to close the app without writing the .gitignore file.";
    siv.add_layer(Dialog::info(message).h_align(HAlign::Center));
}
fn about(siv: &mut Cursive) {
    let mut styled = StyledString::styled("+---------------+\n", BaseColor::Yellow.dark());
    styled.append(StyledString::styled("|", BaseColor::Yellow.dark()));
    styled.append(StyledString::plain(" g i g - g e n "));
    styled.append(StyledString::styled("|\n", BaseColor::Yellow.dark()));
    styled.append(StyledString::styled(
        "+---------------+\n",
        BaseColor::Yellow.dark(),
    ));
    styled.append(StyledString::plain(format!(
        "v {}\n",
        env!("CARGO_PKG_VERSION")
    )));
    styled.append(StyledString::plain("Copyright © 2024 Paul Sobolik\n\n"));
    styled.append(StyledString::plain("API and templates provided by\n"));
    styled.append(StyledString::plain(
        "https://www.toptal.com/developers/gitignore/",
    ));

    siv.add_layer(
        Dialog::around(TextView::new(styled).h_align(HAlign::Center))
            .h_align(HAlign::Center)
            .button("Ok", |s| {
                s.pop_layer();
            }),
    );
}
