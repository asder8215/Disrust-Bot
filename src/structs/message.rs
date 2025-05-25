use serenity::all::CreateAttachment;


#[derive(Debug)]
pub struct Message {
    pub text: String,
    pub attachment: Option<CreateAttachment>,
}