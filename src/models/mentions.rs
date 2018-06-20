use activitypub::link;
use diesel::{self, PgConnection, QueryDsl, RunQueryDsl, ExpressionMethods};

use activity_pub::{Id, inbox::Notify};
use models::{
    comments::Comment,
    notifications::*,
    posts::Post,
    users::User
};
use schema::mentions;

#[derive(Queryable, Identifiable)]
pub struct Mention {
    pub id: i32,
    pub mentioned_id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub ap_url: String
}

#[derive(Insertable)]
#[table_name = "mentions"]
pub struct NewMention {
    pub mentioned_id: i32,
    pub post_id: Option<i32>,
    pub comment_id: Option<i32>,
    pub ap_url: String
}

impl Mention {
    insert!(mentions, NewMention);
    get!(mentions);
    find_by!(mentions, find_by_ap_url, ap_url as String);
    list_by!(mentions, list_for_user, mentioned_id as i32);

    pub fn get_mentioned(&self, conn: &PgConnection) -> Option<User> {
        User::get(conn, self.mentioned_id)
    }

    pub fn get_post(&self, conn: &PgConnection) -> Option<Post> {
        self.post_id.and_then(|id| Post::get(conn, id))
    }

    pub fn get_comment(&self, conn: &PgConnection) -> Option<Comment> {
        self.post_id.and_then(|id| Comment::get(conn, id))
    }

    pub fn to_activity(&self, conn: &PgConnection) -> link::Mention {
        let user = self.get_mentioned(conn);
        let mut mention = link::Mention::default();
        mention.link_props.set_href_string(user.clone().map(|u| u.ap_url).unwrap_or(String::new())).expect("Error setting mention's href");
        mention.link_props.set_name_string(user.map(|u| format!("@{}", u.get_fqn(conn))).unwrap_or(String::new())).expect("Error setting mention's name");
        mention
    }

    pub fn from_activity(conn: &PgConnection, ment: link::Mention, inside: Id) -> Option<Self> {
        let mentioned = User::find_by_ap_url(conn, ment.link_props.href_string().unwrap()).unwrap();

        if let Some(post) = Post::find_by_ap_url(conn, inside.clone().into()) {
            let res = Some(Mention::insert(conn, NewMention {
                mentioned_id: mentioned.id,
                post_id: Some(post.id),
                comment_id: None,
                ap_url: ment.link_props.href_string().unwrap_or(String::new())
            }));
            Mention::notify(conn, ment, Id::new(String::new()));
            res
        } else {
            if let Some(comment) = Comment::find_by_ap_url(conn, inside.into()) {
                let res =Some(Mention::insert(conn, NewMention {
                    mentioned_id: mentioned.id,
                    post_id: None,
                    comment_id: Some(comment.id),
                    ap_url: ment.link_props.href_string().unwrap_or(String::new())
                }));
                Mention::notify(conn, ment, Id::new(String::new()));
                res
            } else {
                None
            }
        }
    }
}

impl Notify<link::Mention> for Mention {
    fn notify(conn: &PgConnection, ment: link::Mention, _actor: Id) {
        match Mention::find_by_ap_url(conn, ment.link_props.href_string().unwrap()) {
            Some(mention) => {
                let author = mention.get_comment(conn)
                    .map(|c| c.get_author(conn).display_name.clone())
                    .unwrap_or(mention.get_post(conn).unwrap().get_authors(conn)[0].display_name.clone());

                mention.get_mentioned(conn).map(|m| {
                    Notification::insert(conn, NewNotification {
                        title: "{{ data }} mentioned you.".to_string(),
                        data: Some(author),
                        content: None,
                        link: Some(mention.get_post(conn).map(|p| p.ap_url).unwrap_or(mention.get_comment(conn).unwrap().ap_url.unwrap_or(String::new()))),
                        user_id: m.id
                    });
                });
            },
            None => println!("Couldn't find mention by AP URL, to create a new notification")
        };
    }
}
