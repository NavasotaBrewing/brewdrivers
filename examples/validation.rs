use brewdrivers::model::validation;
use brewdrivers::model::RTU;

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let mut rtu = RTU::generate().expect("could not read RTU conf from file");

    let result = validation::validate_all(&mut rtu);
    println!("{:?}", result);
}
