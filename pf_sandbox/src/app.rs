#[cfg(feature = "vulkan")]
use ::vulkan::VulkanGraphics;
#[cfg(feature = "opengl")]
use ::opengl::OpenGLGraphics;
#[cfg(any(feature = "vulkan", feature = "opengl"))]
use ::cli::GraphicsBackendChoice;
#[cfg(any(feature = "vulkan", feature = "opengl"))]
use ::graphics::GraphicsMessage;
#[cfg(any(feature = "vulkan", feature = "opengl"))]
use std::sync::mpsc::Sender;
use std;

use ::ai;
use ::cli::{CLIResults, ContinueFrom};
use ::command_line::CommandLine;
use ::config::Config;
use ::game::{Game, GameState, GameSetup};
use ::input::Input;
use ::menu::{Menu, MenuState, ResumeMenu};
use ::network::Network;
use ::os_input::OsInput;
use ::package::Package;
use ::package;

use libusb::Context;
use std::time::{Duration, Instant};

pub fn run(mut cli_results: CLIResults) {
    if let ContinueFrom::Close = cli_results.continue_from {
        return;
    }

    let mut context = Context::new().unwrap();
    let mut input = Input::new(&mut context);
    #[cfg(any(feature = "vulkan", feature = "opengl"))]
    let mut graphics_tx: Option<Sender<GraphicsMessage>> = None;
    let mut network = Network::new();

    // CLI options
    let (mut menu, mut game, mut os_input) = {
        #[allow(unused_variables)] // Needed for headless build
        let (os_input, os_input_tx) = OsInput::new();

        package::generate_example_stub();
        let config = Config::load();
        let package_string = cli_results.package.or(config.current_package.clone());

        #[cfg(any(feature = "vulkan", feature = "opengl"))]
        {
            match cli_results.graphics_backend {
                #[cfg(feature = "vulkan")]
                GraphicsBackendChoice::Vulkan => {
                    graphics_tx = Some(VulkanGraphics::init(os_input_tx.clone()));
                }
                #[cfg(feature = "opengl")]
                GraphicsBackendChoice::OpenGL => {
                    graphics_tx = Some(OpenGLGraphics::init(os_input_tx.clone()));
                }
                GraphicsBackendChoice::Headless => {}
                GraphicsBackendChoice::Default => {
                    #[cfg(feature = "vulkan")]
                    {
                        graphics_tx = Some(VulkanGraphics::init(os_input_tx.clone()));
                    }
                    #[cfg(all(not(feature = "vulkan"), feature = "opengl"))]
                    {
                        graphics_tx = Some(OpenGLGraphics::init(os_input_tx.clone()));
                    }
                }
            }
        }

        let package = if let Some(package_string) = package_string {
            Package::open_or_generate(&package_string)
        } else {
            None
        };
        let menu_state: MenuState = if let Some(_) = package {
            MenuState::character_select()
        } else {
            MenuState::package_select()
        };

        match cli_results.continue_from {
            ContinueFrom::Menu => {
                (
                    Menu::new(package, config, menu_state),
                    None,
                    os_input
                )
            }
            ContinueFrom::Game => {
                // handle no package
                let package = if let Some(package) = package {
                    package
                } else {
                    println!("No package was selected.");
                    println!("As a fallback we tried to use the last used package, but that wasnt available either.");
                    println!("Please select a package.");
                    return;
                };

                // handle issues with package that prevent starting from game
                if package.fighters.len() == 0 {
                    println!("Selected package has no fighters");
                    return;
                }
                else if package.stages.len() == 0 {
                    println!("Selected package has no stages");
                    return;
                }

                // handle missing and invalid cli input
                for name in &cli_results.fighter_names {
                    if !package.fighters.contains_key(name) {
                        println!("Package does not contain selected fighter '{}'", name);
                        return;
                    }
                }
                if let &Some(ref name) = &cli_results.stage_name {
                    if !package.stages.contains_key(name) {
                        println!("Package does not contain selected stage '{}'", name);
                        return;
                    }
                }

                // handle missing and invalid cli input
                if cli_results.fighter_names.len() == 0 {
                    cli_results.fighter_names.push(package.fighters.index_to_key(0).unwrap());
                }

                // fill fighters/controllers
                let mut controllers: Vec<usize> = vec!();
                let mut fighters: Vec<String> = vec!();
                input.update(&[], &[]); // run the first input step so that we can check for the number of controllers.
                for (i, _) in input.players(0).iter().enumerate() {
                    controllers.push(i);
                    fighters.push(cli_results.fighter_names[i % cli_results.fighter_names.len()].clone());
                }

                // remove extra fighters/controllers
                if let Some(max_players) = cli_results.max_human_players {
                    while controllers.len() > max_players {
                        controllers.pop();
                        fighters.pop();
                    }
                }

                // add cpu players
                let mut ais: Vec<usize> = vec!();
                let last_fighter_i = fighters.len();
                if let Some(total_players) = cli_results.total_cpu_players {
                    for i in 0..total_players {
                        fighters.push(cli_results.fighter_names[(last_fighter_i + i) % cli_results.fighter_names.len()].clone());
                        controllers.push(last_fighter_i + i);
                        ais.push(0);
                    }
                }

                if cli_results.stage_name.is_none() {
                    cli_results.stage_name = package.stages.index_to_key(0);
                }

                let setup = GameSetup {
                    init_seed:      GameSetup::gen_seed(),
                    input_history:  vec!(),
                    player_history: vec!(),
                    stage_history:  vec!(),
                    controllers:    controllers,
                    fighters:       fighters,
                    ais:            ais,
                    stage:          cli_results.stage_name.unwrap(),
                    state:          GameState::Local,
                };
                (
                    Menu::new(None, config.clone(), menu_state),
                    Some(Game::new(package, config, setup)),
                    os_input
                )
            }
            _ => unreachable!()
        }
    };

    let mut command_line = CommandLine::new();

    loop {
        let frame_start = Instant::now();

        os_input.update();

        let mut resume_menu: Option<ResumeMenu> = None;
        if let Some(ref mut game) = game {
            let ai_inputs = ai::gen_inputs(&game);
            input.update(&game.tas, &ai_inputs);
            if let GameState::Quit (resume_menu_inner) = game.step(&mut input, &os_input, command_line.block()) {
                resume_menu = Some(resume_menu_inner)
            }
            #[cfg(any(feature = "vulkan", feature = "opengl"))]
            {
                if let Some(ref tx) = graphics_tx {
                    if let Err(_) = tx.send(game.graphics_message(&command_line)) {
                        return;
                    }
                }
            }
            network.update(game);
            command_line.step(&os_input, game);
        }
        else {
            input.update(&[], &[]);
            if let Some(mut menu_game_setup) = menu.step(&mut input) {
                let (package, config) = menu.reclaim();
                input.set_history(std::mem::replace(&mut menu_game_setup.input_history, vec!()));
                game = Some(Game::new(package, config, menu_game_setup));
            }
            else {
                #[cfg(any(feature = "vulkan", feature = "opengl"))]
                {
                    if let Some(ref tx) = graphics_tx {
                        if let Err(_) = tx.send(menu.graphics_message(&command_line)) {
                            return;
                        }
                    }
                }
            }
            network.update(&mut menu);
            command_line.step(&os_input, &mut menu);
        }

        if let Some(resume_menu) = resume_menu {
            let (package, config) = match game {
                Some (game) => game.reclaim(),
                None        => unreachable!()
            };
            input.reset_history();
            game = None;
            menu.resume(package, config, resume_menu);

            // Game -> Menu Transitions
            // Game complete   -> display results -> CSS
            // Game quit       -> CSS
            // Replay complete -> display results -> replay screen
            // Replay quit     -> replay screen
        }

        if os_input.quit() {
            return;
        }

        let frame_duration = Duration::from_secs(1) / 60;
        while frame_start.elapsed() < frame_duration { }
    }
}
