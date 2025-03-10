use clap::Parser;
use habitcli::{
    args::{self, IntoArguments, ParsedArguments},
    pixela::Session, Worker, error::Error
};

fn main() -> Result<(), Error> {
    let args = args::HabitCLIArgs::parse();

    let session = Session::new();
    let mut worker = Worker::new(session);
// match statement for every possible user inputted command
    match args.command_type {
        args::CommandType::Signup(arguments) => {
            if let ParsedArguments::NewUserData(args) = arguments.into_args() {
                worker.call_create_user(args)?;
            }
        }
        args::CommandType::Send(arguments) => {
            worker.login()?;
            if let ParsedArguments::PixelArgs(args) = arguments.into_args() {
                worker.call_send(args);
            }
        }

        args::CommandType::Get(arguments) => {
            worker.login()?;
            if let ParsedArguments::PixelArgs(args) = arguments.into_args() {
                worker.call_get(args)
            }
        }

        args::CommandType::Login(arguments) => {
            if let ParsedArguments::LoginArgs(args) = arguments.into_args() {
                match worker.call_save_data(args) {
                    Ok(_) => (),
                    Err(e) => println!("{:?}", e),
                }
            }
        }
        args::CommandType::Create(arguments) => {
            worker.login()?;
            if let ParsedArguments::GraphCreateArgs(args) = arguments.into_args() {
                worker.call_create_graph(args)?;
            }
        }
        args::CommandType::List(_) => {
            worker.login()?;
            worker.call_list()?;
        }
        args::CommandType::Streak(arguments) => {
            worker.login()?;
            if let ParsedArguments::StreakGetArgs(args) = arguments.into_args() {
                worker.call_streak(args)?
            }
        }
        args::CommandType::SetupSum(arguments) => {
            if let ParsedArguments::SumGraphArgs(args) = arguments.into_args()  {
                match worker.call_save_sum_graph(args){
                    Ok(_) => (),
                    Err(e) => println!("{:?}",e)
                }
            }
        }
        args::CommandType::Data(_) => {
            worker.login()?;
            worker.print_data()?;
        }

        args::CommandType::Sum(arguments) => {
            worker.login()?;
            if let ParsedArguments::SumArgs(args) = arguments.into_args() {
                match worker.handle_sum_graph(args) {
                    Ok(_) => (),
                    Err(e) => println!("{:?}",e)
                }
            worker.login()?;
    
        }
        }

    };
    Ok(())
}
