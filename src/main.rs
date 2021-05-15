use std::thread::sleep;
use std::time::Duration;
use chrono::prelude::Locale;
use chrono::Local;
use cmd_lib::run_fun;
use std::error::Error;
use std::process;

const ONE_SECOND: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        run()?;
        sleep(ONE_SECOND);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let player = mpris::PlayerFinder::new().unwrap_or_else(|_| {
        eprintln!("Could not get Dbus connection");
        process::exit(1);
    });
    let thing_playing = if let Ok(p) = player.find_active() {
        parse_player(&p)
    } else {
        "No players found".to_owned()
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
    let pactl_out = run_fun!(pactl list sinks)?;
    let vol_line = pactl_out
        .lines()
        .filter(|line| line.contains("Volume"))
        .collect::<Vec<&str>>();
    let vol = vol_line
        .first()
        .unwrap()
        .split_whitespace()
        .filter(|x| x.ends_with("%"))
        .collect::<Vec<&str>>();
    let vol_level = vol.first().unwrap();
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

fn parse_player(player: &mpris::Player) -> String {
    let status = player.get_playback_status();
    let metadata = player.get_metadata().unwrap();
    let artist = if let Some(artists) = metadata.artists() {
        artists.get(0).unwrap()
    } else {
        ""
    };
    let title = metadata.title().unwrap_or_default();
    let song = if artist.is_empty() && title.is_empty() {
        "Something is playing...".to_owned()
    } else {
        format!("{} — {}", artist, title)
    };
    match status {
        Ok(mpris::PlaybackStatus::Paused) | Ok(mpris::PlaybackStatus::Stopped) => {
            format!("({:?}) {}", status.unwrap(), song)
        }
        _ => song,
    }
}
