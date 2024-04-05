use brewdrivers::model::validation;
use brewdrivers::model::{conditions::ConditionCollection, rules::RuleSet, RTU};

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("trace"));

    let rtu = RTU::generate().expect("could not read RTU conf from file");
    let conditions =
        ConditionCollection::generate().expect("could not read condition conf from file");
    let rules = RuleSet::generate().expect("could not read rules conf from file");

    let result = validation::validate_all(&rtu, &conditions, &rules);
    println!("{:?}", result);
}
