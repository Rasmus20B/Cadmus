
mod dev_db;

use tokio::sync::OnceCell;

use crate::model::ModelManager;

pub async fn init_dev() {
   static INIT: OnceCell<()> = OnceCell::const_new();

    INIT.get_or_init(|| async {
        dev_db::init_dev_db().await.unwrap();
    })
    .await;
}

pub async fn init_test() -> ModelManager {
    static INIT: OnceCell<ModelManager> = OnceCell::const_new();

    let mm = INIT
        .get_or_init(|| async {
            init_dev().await;
            ModelManager::new().await.unwrap()
        })
    .await;

    mm.clone()
}
