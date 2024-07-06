use uuid::Uuid;

pub fn get_random_name() -> String {
    let uuid = Uuid::new_v4();
    String::from(uuid)
}
