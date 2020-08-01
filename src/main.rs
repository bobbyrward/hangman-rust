use std::process::Command;
use unicode_width::UnicodeWidthStr;

use cursive::align::HAlign;
use cursive::traits::*;
use cursive::views::{Canvas, Dialog, EditView, LinearLayout, Panel, TextView};

const ALPHABET: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];
const INVALID_INPUT: u8 = 255;
const HANGMAN: [&str; 7] = [
    " ╔═══╗\n ║    \n ║\n ║\n═╩══",
    " ╔═══╗\n ║   O\n ║\n ║\n═╩══",
    " ╔═══╗\n ║   O\n ║   |\n ║\n═╩══",
    " ╔═══╗\n ║   O\n ║  /|\n ║\n═╩══",
    " ╔═══╗\n ║   O\n ║  /|\\\n ║\n═╩══",
    " ╔═══╗\n ║   O\n ║  /|\\\n ║  /\n═╩══",
    " ╔═══╗\n ║   O\n ║  /|\\\n ║  / \\\n═╩══",
];
const MAX_MISSES: usize = 6;
const CENTER_OFFSET: i32 = 6;

#[derive(Clone, PartialEq)]
enum GameState {
    Running {
        secret: String,
        misses: usize,
        guesses: [bool; 26],
    },
    Lost,
    Won,
}

impl GameState {
    fn new() -> Self {
        Self::Running {
            misses: 0,
            secret: HangmanGame::generate_word().to_uppercase(),
            guesses: [false; 26],
        }
    }

    pub fn is_running(&self) -> bool {
        match self {
            Self::Running { .. } => true,
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn is_lost(&self) -> bool {
        match self {
            Self::Lost => true,
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn is_won(&self) -> bool {
        match self {
            Self::Won => true,
            _ => false,
        }
    }
}

struct HangmanGame {
    state: GameState,
    canvas: String,
}

fn char_to_index(c: char) -> usize {
    let i = c as u8;
    if i < 65 || i > 90 {
        INVALID_INPUT as usize
    } else {
        (i - 65) as usize
    }
}

impl HangmanGame {
    fn new_game() -> HangmanGame {
        HangmanGame {
            canvas: String::new(),
            state: GameState::new(),
        }
    }

    fn solved_word(&self) -> bool {
        match self.state {
            GameState::Running {
                ref secret,
                ref guesses,
                ..
            } => !secret.chars().any(|c| !guesses[char_to_index(c)]),
            _ => panic!("solved_word on a finished game"),
        }
    }

    fn letter_store(&self) -> String {
        match self.state {
            GameState::Running { ref guesses, .. } => guesses
                .iter()
                .enumerate()
                .map(|(i, guessed)| {
                    format!(
                        "{}{}",
                        if i == 13 { "\n" } else { "" },
                        if *guessed { ALPHABET[i as usize] } else { ' ' },
                    )
                })
                .collect(),
            _ => panic!("letter_store on finished game"),
        }
    }

    fn enter_letter(&mut self, guess: String) -> Option<String> {
        match self.state {
            GameState::Running {
                ref mut guesses,
                ref secret,
                ref mut misses,
                ..
            } => {
                let index = match HangmanGame::guess_to_index(&guess) {
                    Some(index) => index,
                    None => return None,
                };

                // React to guess based on state
                if guesses[index as usize] {
                    return None;
                } else if secret.contains(&guess.to_uppercase()) {
                    // Guess was in word
                } else if *misses < MAX_MISSES {
                    *misses += 1;
                }
                let secret = secret.clone();

                // Update state
                guesses[index as usize] = true;
                self.canvas = String::from(HANGMAN[*misses]);

                // Check if the game is over
                if *misses == MAX_MISSES {
                    self.state = GameState::Lost;
                } else if self.solved_word() {
                    self.state = GameState::Won;
                }

                Some(secret)
            }
            _ => panic!("enter_letter on a finished game"),
        }
    }

    fn get_status(&self) -> String {
        match self.state {
            GameState::Running {
                ref guesses,
                ref secret,
                ..
            } => secret
                .chars()
                .map(|c| if guesses[char_to_index(c)] { c } else { '_' })
                .collect(),
            _ => panic!("get_status on a finished game"),
        }
    }

    fn guess_to_index(guess: &str) -> Option<u8> {
        guess
            .trim()
            .to_uppercase()
            .parse::<char>()
            .ok()
            .map(|c| char_to_index(c) as u8)
    }

    fn generate_word() -> String {
        let o = Command::new("shuf")
            .arg("/usr/share/dict/american-english")
            .output()
            .expect("error")
            .stdout;

        String::from_utf8(o)
            .expect("word should be valid utf-8")
            .lines()
            .find(|w| !w.contains('\''))
            .expect("Should have found at least one word")
            .to_string()
    }

    #[allow(dead_code)]
    fn secret(&self) -> Option<&String> {
        match self.state {
            GameState::Running { ref secret, .. } => Some(secret),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn misses(&self) -> Option<usize> {
        match self.state {
            GameState::Running { misses, .. } => Some(misses),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn guesses(&self) -> Option<&[bool]> {
        match self.state {
            GameState::Running { ref guesses, .. } => Some(guesses),
            _ => None,
        }
    }

    fn state(&self) -> &GameState {
        &self.state
    }
}

fn game_over(s: &mut cursive::Cursive, result: GameState, word: String) {
    // Get end message
    let message = match result {
        GameState::Won => "Congratulations! You won!",
        GameState::Lost => "You lost! Better luck next time :(",
        _ => panic!("Something has gone terribly wrong..."),
    };

    let message_view = TextView::new(message).h_align(HAlign::Center);
    let word_view = Panel::new(TextView::new(word).h_align(HAlign::Center)).title("Solution");

    s.pop_layer();
    s.add_layer(
        Dialog::new()
            .title("Hangman")
            .content(
                LinearLayout::vertical()
                    .child(word_view)
                    .child(message_view),
            )
            .button("Quit", |s| s.quit()),
    );
}

fn game_tick(s: &mut cursive::Cursive, guess: &str) {
    // Clear input field
    s.call_on_name("input", |view: &mut EditView| view.set_content(""));

    let game: &mut HangmanGame = s
        .user_data()
        .expect("Should never get here.  Data should always be present.");

    let mut secret = String::new();
    if let Some(end_secret) = game.enter_letter(String::from(guess)) {
        secret = end_secret;
    }

    if game.state().is_running() {
        let store = game.letter_store();
        let status = game.get_status();
        let misses = game.misses().expect("Should be running");

        s.call_on_name("canvas", |view: &mut Canvas<String>| {
            view.set_draw(move |_, printer| {
                HANGMAN[misses]
                    .lines()
                    .enumerate()
                    .for_each(|(i, l)| printer.print((CENTER_OFFSET, i as i32), l));
            })
        });
        s.call_on_name("letter_store", |view: &mut TextView| {
            view.set_content(store);
        });
        s.call_on_name("status", |view: &mut TextView| {
            view.set_content(status);
        });
    } else {
        let state = game.state.clone();

        game_over(s, state, secret);
    }
}

fn build_ui() -> cursive::Cursive {
    let mut ui = cursive::default();
    let game = HangmanGame::new_game();
    let secret_status = game.get_status();
    let store = game.letter_store();

    ui.set_user_data(game);

    let input_view = EditView::new().on_submit(game_tick).with_name("input");

    let canvas_state = String::from(HANGMAN[0]);
    let hangman_view = Canvas::new(canvas_state)
        .with_draw(|state, printer| {
            let lines = state.lines();
            for (i, l) in lines.enumerate() {
                printer.print((CENTER_OFFSET, i as i32), l);
            }
        })
        .with_required_size(|text, _constraints| (text.width(), 7).into())
        .with_name("canvas");

    let letter_store = Panel::new(
        TextView::new(store)
            .h_align(HAlign::Center)
            .with_name("letter_store"),
    )
    .title("Guesses");

    let word_display = TextView::new(secret_status)
        .h_align(HAlign::Center)
        .with_name("status");

    ui.add_layer(
        Dialog::new()
            .title("Hangman!")
            .content(
                LinearLayout::vertical()
                    .child(hangman_view)
                    .child(word_display)
                    .child(input_view)
                    .child(letter_store),
            )
            .button("Quit", |s| s.quit()),
    );
    ui
}

fn main() {
    build_ui().run();
}
