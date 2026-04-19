use clap::Parser;
use std::{
    io,
    net::{IpAddr, SocketAddr},
    thread::sleep,
    time::Duration,
};
use tabled::{
    Table, Tabled,
    settings::{
        Alignment, Color, Panel, Style,
        object::Rows,
        style::BorderColor,
        themes::{BorderCorrection, Colorization},
    },
};

#[derive(Parser, Debug)]
struct Args {
    ip: String,

    /// Port to use
    #[arg(long, default_value_t = 27015)]
    port: u16,

    /// Join when less than this amount of players
    #[arg(short, long, default_value_t = 62)]
    players: u8,

    /// How often to check if there's a slot available
    #[arg(short, long, default_value_t = 1)]
    check: u8,
}

#[derive(Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct Player {
    /// Player's name.
    pub name: String,
    /// Player's score.
    pub score: i32,
    /// How long a player has been in the server (seconds).
    #[tabled(display = "display_time")]
    pub duration: f32,
}

#[derive(Tabled)]
#[tabled(rename_all = "PascalCase")]
pub struct Server {
    pub server: String,
    pub map: String,
    pub players: u8,
}

fn display_time(o: &f32) -> String {
    let time = *o as i32;

    format!("{:02}:{:02}", time / 60, time % 60)
}

use gamedig::{
    TimeoutSettings,
    protocols::{
        types::GatherToggle,
        valve::{self, Engine, GatheringSettings},
    },
};

fn main() {
    let argv = Args::parse();
    let ip: IpAddr = (&argv.ip).parse().expect("valid IP");
    let address = &SocketAddr::new(ip, argv.port);

    let engine = Engine::Source(Some((730, None))); // We don't specify a steam app id, let the query try to find it.
    let gather_settings = GatheringSettings {
        players: GatherToggle::Enforce, // We want to query for players
        rules: GatherToggle::Skip,      // We don't want to query for rules
        check_app_id: false,            // Loosen up the query a bit by not checking app id
    };

    let read_timeout = Duration::from_secs(1);
    let write_timeout = Duration::from_secs(1);
    let connect_timeout = Duration::from_secs(1);
    let retries = 1; // does another request if the first one fails.
    let timeout_settings = TimeoutSettings::new(
        Some(read_timeout),
        Some(write_timeout),
        Some(connect_timeout),
        retries,
    )
    .unwrap();

    #[inline]
    fn style_table(table: &mut Table, header: &String, footer: &String) {
        table
            .with(Style::sharp())
            .with(Panel::header(header))
            .with(Panel::footer(footer))
            .with(Alignment::center())
            .with(BorderCorrection::span())
            .with(Colorization::rows([
                Color::BG_BLUE | Color::FG_CYAN,
                Color::BG_CYAN | Color::FG_BLUE,
            ]))
            .with(BorderColor::filled(Color::BG_BLACK));

        table
            .modify(Rows::first(), Color::FG_WHITE)
            .modify(Rows::first(), Color::BG_BRIGHT_BLACK)
            .modify(Rows::last(), Color::FG_WHITE)
            .modify(Rows::last(), Color::BG_BRIGHT_BLACK);
    }

    #[inline]
    fn format_response(r: &gamedig::protocols::valve::Response) -> String {
        format!(
            "{map} ({players_online}/{players_maximum})",
            map = r.info.map,
            players_online = r.info.players_online,
            players_maximum = r.info.players_maximum,
        )
    }

    let mut error_count = 0;
    loop {
        match valve::query(
            address,
            engine,
            Some(gather_settings),
            Some(timeout_settings),
        ) {
            Ok(response) => {
                let mut players: Vec<Player> = vec![];

                if let Some(players_details) = &response.players {
                    players = players_details
                        .iter()
                        .map(|player| Player {
                            name: player.name.clone(),
                            score: player.score,
                            duration: player.duration,
                        })
                        .collect();
                }

                players.sort_by_key(|p| {
                    // Assuming empty player names are connecting, move them to the bottom
                    if p.name.is_empty() {
                        0
                    } else {
                        -(p.duration as i32)
                    }
                });

                let _ = clearscreen::clear();
                let mut table = Table::new(players);
                style_table(&mut table, &response.info.name, &format_response(&response));

                println!("{}", table);
                if response.info.players_online < argv.players {
                    open::that(format!(
                        "steam://rungame/730/76561202255233023/+connect%20{}:{}",
                        argv.ip, argv.port
                    ))
                    .expect("we can open steam links");

                    print!("\x07");
                    println!("Press [enter] to continue:");
                    let mut _input: String = String::default();
                    let _ = io::stdin().read_line(&mut _input);
                }
            }
            Err(err) => {
                println!("error: {}", err);

                error_count += 1;
                sleep(Duration::from_secs(error_count));
            }
        }

        sleep(Duration::from_secs(argv.check.into()));
    }
}
