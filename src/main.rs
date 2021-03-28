use chrono::prelude::Locale;
use chrono::Local;
use cmd_lib::run_fun;
use std::error::Error;
use std::process;

fn main() -> Result<(), Box<dyn Error>> {
    let player = mpris::PlayerFinder::new()
        .unwrap_or_else(|_| {
            eprintln!("Could not get Dbus connection");
            process::exit(1);
        })
        .find_active();
    let thing_playing = match player {
        Ok(player) => {
            let status = player.get_playback_status();
            let metadata = player.get_metadata().unwrap();
            let song = format!(
                "{} — {}",
                metadata.artists().unwrap().get(0).unwrap(),
                metadata.title().unwrap()
            );
            match status {
                Ok(mpris::PlaybackStatus::Paused) | Ok(mpris::PlaybackStatus::Stopped) => {
                    format!("({:?}) {}", status.unwrap(), song)
                }
                _ => song,
            }
        }
        _ => "No players found".to_owned(),
    };
    let now = Local::now().format_localized("%a %d %b %H:%M %p", Locale::fr_FR);
    let bats = battery::Manager::new()?
        .batteries()?
        .map(|bat| bat.unwrap())
        .map(|bat| (bat.state(), bat.state_of_charge() * 100f32))
        .collect::<Vec<(battery::State, battery::units::Ratio)>>();
    let mut bat_state = String::from("");
    if bats.len() == 2 {
        let bat_one = &bats[0];
        let bat_two = &bats[1];
        bat_state = format!(
            "{} {}% {}%",
            bat_two.0,
            bat_two.1.value.round(),
            bat_one.1.value.round()
        );
    }
    let amixer_out = run_fun!(amixer -D pulse sget Master)?;
    let vol = amixer_out
        .lines()
        .last()
        .unwrap()
        .split_whitespace()
        .filter(|x| x.starts_with("["))
        .collect::<Vec<&str>>();
    let vol_level = vol[0].replace(|c| c == '[' || c == ']', "");
    let components = vec![
        thing_playing,
        String::from("♪ ") + &vol_level,
        bat_state,
        now.to_string(),
    ];

    println!("{}", make_status(&components));
    Ok(())
}

fn make_status<S: AsRef<str>>(components: &Vec<S>) -> String {
    let separator = " • ";
    components
        .iter()
        .map(|c| c.as_ref())
        .filter(|c| !c.is_empty())
        .collect::<Vec<&str>>()
        .join(separator)
}
