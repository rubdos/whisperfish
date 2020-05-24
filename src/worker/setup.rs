use std::rc::Rc;

use qmetaobject::*;
use failure::*;

use crate::gui::WhisperfishApp;
use crate::store::{self, Storage};

#[derive(QObject, Default)]
#[allow(non_snake_case)]
pub struct SetupWorker {
    base: qt_base_class!(trait QObject),

    registrationSuccess: qt_signal!(),
    invalidDatastore: qt_signal!(),
    invalidPhoneNumber: qt_signal!(),
    clientFailed: qt_signal!(),
    setupComplete: qt_signal!(),

    phoneNumber: qt_property!(QString; NOTIFY setupChanged),
    registered: qt_property!(bool; NOTIFY setupChanged),
    locked: qt_property!(bool; NOTIFY setupChanged),
    encryptedKeystore: qt_property!(bool; NOTIFY setupChanged),
    localId: qt_property!(QString; NOTIFY setupChanged),
    identity: qt_property!(QString; NOTIFY setupChanged),

    /// Emitted when any of the properties change.
    setupChanged: qt_signal!(),
}

impl SetupWorker {
    pub async fn run(app: Rc<WhisperfishApp>) {
        log::info!("SetupWorker::run");
        let this = app.setup_worker.pinned();

        // Check registration

        // Open storage
        let _storage = match SetupWorker::setup_storage(app.clone()).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("Error setting up storage: {}", e);
                this.borrow().clientFailed();
                return;
            }
        };
    }

    async fn setup_storage(app: Rc<WhisperfishApp>) -> Result<Storage, Error> {
        let settings = app.settings.pinned();

        if settings.borrow().get_bool("encrypt_database") {
            let password = app.prompt.pinned().borrow_mut().ask_password().await;
            bail!("Decrypting not yet implemented")
        } else {
            Ok(Storage::open(&store::default_location()?)?)
        }
    }
}
