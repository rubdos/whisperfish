use crate::gui::StorageReady;
use crate::model::message::MessageModel;
use crate::sfos::SailfishApp;
use crate::store::Storage;
use crate::worker::ClientActor;

use actix::prelude::*;
use qmetaobject::*;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FetchSession {
    pub id: i64,
    pub mark_read: bool,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FetchMessage(pub i64);

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FetchAllMessages(pub i64);

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct DeleteMessage(pub i64, pub usize);

#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct QueueMessage {
    pub source: String,
    pub message: String,
    pub attachment: String,
    pub group: String,
}

pub struct MessageActor {
    inner: QObjectBox<MessageModel>,
    storage: Option<Storage>,
}

impl MessageActor {
    pub fn new(app: &mut SailfishApp, client: Addr<ClientActor>) -> Self {
        let inner = QObjectBox::new(MessageModel::default());
        app.set_object_property("MessageModel".into(), inner.pinned());
        inner.pinned().borrow_mut().client_actor = Some(client);

        Self {
            inner,
            storage: None,
        }
    }
}

impl Actor for MessageActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.inner.pinned().borrow_mut().actor = Some(ctx.address());
    }
}

impl Handler<StorageReady> for MessageActor {
    type Result = ();

    fn handle(
        &mut self,
        StorageReady(storage, _config): StorageReady,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.storage = Some(storage);
        log::trace!("MessageActor has a registered storage");
    }
}

impl Handler<FetchSession> for MessageActor {
    type Result = ();

    fn handle(
        &mut self,
        FetchSession { id: sid, mark_read }: FetchSession,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let storage = self.storage.as_ref().unwrap();
        let mut sess = storage
            .fetch_session(sid)
            .expect("FIXME No session returned!");
        if mark_read {
            storage.mark_session_read(&sess);
            sess.unread = false;
        }
        self.inner.pinned().borrow_mut().handle_fetch_session(sess);
    }
}

impl Handler<FetchMessage> for MessageActor {
    type Result = ();

    fn handle(&mut self, FetchMessage(id): FetchMessage, _ctx: &mut Self::Context) -> Self::Result {
        let message = self.storage.as_ref().unwrap().fetch_message(id);
        self.inner
            .pinned()
            .borrow_mut()
            .handle_fetch_message(message.expect("FIXME No message returned!"));
    }
}

impl Handler<FetchAllMessages> for MessageActor {
    type Result = ();

    fn handle(
        &mut self,
        FetchAllMessages(sid): FetchAllMessages,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let messages = self.storage.as_ref().unwrap().fetch_all_messages(sid);
        self.inner
            .pinned()
            .borrow_mut()
            .handle_fetch_all_messages(messages.expect("FIXME No messages returned!"));
    }
}

impl Handler<DeleteMessage> for MessageActor {
    type Result = ();

    fn handle(
        &mut self,
        DeleteMessage(id, idx): DeleteMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let del_rows = self.storage.as_ref().unwrap().delete_message(id);
        self.inner.pinned().borrow_mut().handle_delete_message(
            id,
            idx,
            del_rows.expect("FIXME no rows deleted"),
        );
    }
}

impl Handler<QueueMessage> for MessageActor {
    type Result = ();

    fn handle(&mut self, msg: QueueMessage, _ctx: &mut Self::Context) -> Self::Result {
        log::trace!("MessageActor::handle({:?})", msg);
        assert!(&msg.group == "", "Creating a new group is unimplemented");

        let storage = self.storage.as_mut().unwrap();

        let has_attachment = !msg.attachment.is_empty();

        let (msg, _session) = storage.process_message(
            crate::store::NewMessage {
                session_id: None,
                source: msg.source,
                text: msg.message,
                timestamp: chrono::Utc::now().naive_utc(),
                has_attachment,
                mime_type: if has_attachment {
                    Some(
                        mime_guess::from_path(&msg.attachment)
                            .first_or_octet_stream()
                            .essence_str()
                            .into(),
                    )
                } else {
                    None
                },
                attachment: if has_attachment {
                    Some(msg.attachment)
                } else {
                    None
                },
                flags: 0,
                outgoing: true,
                received: false,
                sent: false,
            },
            None,
            false,
        );

        storage.queue_message(&msg);

        self.inner.pinned().borrow_mut().handle_queue_message(msg);
    }
}
