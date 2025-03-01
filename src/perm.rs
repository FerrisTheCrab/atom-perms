use std::collections::BTreeMap;

use mongodb::bson::{doc, Bson, Document};

use crate::PermsInstance;

macro_rules! bad_key {
    () => {
        mongodb::error::Error::custom("bad key".to_string())
    };
}

macro_rules! cond_bad_key {
    ($x: expr) => {
        if $x == "_id" || $x.is_empty() {
            return Err(bad_key!());
        }
    };
}

pub struct Perms;

impl Perms {
    fn encode(s: &str) -> String {
        let mut out = String::new();

        for c in s.chars() {
            match c {
                '$' => out.push_str("$d"),
                '.' => out.push_str("$p"),
                c => out.push(c),
            }
        }

        out
    }

    fn decode(s: &str) -> String {
        let mut out = String::new();
        let mut op = false;

        for c in s.chars() {
            match c {
                '$' => op = true,
                c if op => {
                    match c {
                        'd' => out.push('$'),
                        'p' => out.push('.'),
                        _ => panic!("unknown escape sequence ${c}"),
                    };
                    op = false
                }
                c => out.push(c),
            }
        }

        out
    }

    async fn set_int(
        instance: &PermsInstance,
        id: &str,
        entries: Vec<(String, u32)>,
    ) -> Result<(), mongodb::error::Error> {
        let mut m_set = Document::new();
        let mut m_unset = Document::new();

        for (k, v) in entries.into_iter() {
            cond_bad_key!(k);
            if v == 0 {
                m_unset.insert(Self::encode(&k), Bson::Int64(0));
            } else {
                m_set.insert(Self::encode(&k), Bson::Int64(v as i64));
            }
        }

        if instance
            .perms
            .update_one(
                doc! { "_id": id },
                doc! { "$set": m_set.clone(), "$unset": m_unset},
            )
            .await?
            .matched_count
            == 1
        {
            Ok(())
        } else {
            m_set.insert("_id", id);
            instance.perms.insert_one(m_set).await?;
            Ok(())
        }
    }

    async fn wipe_int(instance: &PermsInstance, id: &str) -> Result<(), mongodb::error::Error> {
        instance
            .perms
            .delete_one(doc! {"_id": id})
            .await?;
        Ok(())
    }

    async fn get_int(
        instance: &PermsInstance,
        id: &str,
        entries: Vec<String>,
    ) -> Result<BTreeMap<String, u32>, mongodb::error::Error> {
        let mut projection = Document::new();

        for k in entries.into_iter() {
            cond_bad_key!(k);
            projection.insert(Self::encode(&k), 1);
        }

        let mut entry = instance
            .perms
            .find_one(doc! { "_id": id})
            .projection(projection)
            .await?
            .unwrap_or_default();

        entry.remove("_id");

        let mut out = BTreeMap::new();

        for (k, v) in entry.into_iter() {
            out.insert(Self::decode(&k), v.as_i64().unwrap_or_default() as u32);
        }

        Ok(out)
    }

    async fn list_int(
        instance: &PermsInstance,
        id: &str,
    ) -> Result<BTreeMap<String, u32>, mongodb::error::Error> {
        let mut entry = instance
            .perms
            .find_one(doc! {"_id": id})
            .await?
            .unwrap_or_default();
        entry.remove("_id");

        let mut out = BTreeMap::new();

        for (k, v) in entry.into_iter() {
            out.insert(Self::decode(&k), v.as_i64().unwrap() as u32);
        }

        Ok(out)
    }
}

impl Perms {
    pub async fn set(
        instance: &PermsInstance,
        id: &str,
        entries: Vec<(String, u32)>,
    ) -> Result<(), mongodb::error::Error> {
        Self::set_int(instance, id, entries).await
    }

    pub async fn get(
        instance: &PermsInstance,
        id: &str,
        entries: Vec<String>,
    ) -> Result<BTreeMap<String, u32>, mongodb::error::Error> {
        Self::get_int(instance, id, entries).await
    }

    pub async fn list(
        instance: &PermsInstance,
        id: &str,
    ) -> Result<BTreeMap<String, u32>, mongodb::error::Error> {
        Self::list_int(instance, id).await
    }

    pub async fn wipe(instance: &PermsInstance, id: &str) -> Result<(), mongodb::error::Error> {
        Self::wipe_int(instance, id).await
    }
}
