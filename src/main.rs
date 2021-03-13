use chrono::prelude::Locale;
use chrono::Local;
use cmd_lib::run_fun;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let player = mpris::PlayerFinder::new()
        .expect("Could not connect to D-Bus")
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
        _ => String::from("No player found"),
    };
    let now = Local::now().format_localized("%a %d %b %H:%M %p", Locale::fr_FR);
    let bats = battery::Manager::new()?
        .batteries()?
        .map(|bat| bat.unwrap())
        .map(|bat| (bat.state(), bat.state_of_charge() * 100f32))
        .collect::<Vec<(battery::State, battery::units::Ratio)>>();
    let bat_one = &bats[0];
    let bat_two = &bats[1];
    let amixer_out = run_fun!(amixer -D pulse sget Master)?;
    let vol = amixer_out
        .lines()
        .last()
        .unwrap()
        .split_whitespace()
        .filter(|x| x.starts_with("["))
        .collect::<Vec<&str>>();
    let vol_level = vol[0].replace(|c| c == '[' || c == ']', "");

    println!(
        "{} • ♪ {} • {} {}% {}% • {}",
        thing_playing,
        vol_level,
        bat_two.0,
        bat_two.1.value.round(),
        bat_one.1.value.round(),
        now
    );
    Ok(())
}
