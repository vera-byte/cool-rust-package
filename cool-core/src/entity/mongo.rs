//! MongoDB 实体支持
//!
//! 对应 TypeScript 版本的 `entity/mongo.ts`
//!
//! 提供 MongoDB 实体基类和字段定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for ObjectId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_hex())
    }
}

impl<'de> Deserialize<'de> for ObjectId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        bson::oid::ObjectId::parse_str(&s)
            .map(ObjectId)
            .map_err(serde::de::Error::custom)
    }
}

/// MongoDB ObjectID
///
/// 对应 TypeScript 版本的 `ObjectID`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectId(bson::oid::ObjectId);

impl ObjectId {
    /// 创建新的 ObjectID
    pub fn new() -> Self {
        Self(bson::oid::ObjectId::new())
    }

    /// 从字符串解析 ObjectID
    pub fn parse(s: &str) -> Result<Self, bson::oid::Error> {
        Ok(Self(bson::oid::ObjectId::parse_str(s)?))
    }

    /// 转换为字符串
    pub fn to_string(&self) -> String {
        self.0.to_hex()
    }

    /// 获取内部的 ObjectId
    pub fn inner(&self) -> &bson::oid::ObjectId {
        &self.0
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<bson::oid::ObjectId> for ObjectId {
    fn from(id: bson::oid::ObjectId) -> Self {
        Self(id)
    }
}

impl From<ObjectId> for bson::oid::ObjectId {
    fn from(id: ObjectId) -> Self {
        id.0
    }
}

/// MongoDB 基础实体字段
///
/// 对应 TypeScript 版本的 `BaseMongoEntity`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BaseMongoFields {
    /// ID（ObjectID）
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// 创建时间
    #[serde(rename = "createTime")]
    pub create_time: DateTime<Utc>,
    /// 更新时间
    #[serde(rename = "updateTime")]
    pub update_time: DateTime<Utc>,
}

impl BaseMongoFields {
    /// 创建新的基础字段
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            id: ObjectId::new(),
            create_time: now,
            update_time: now,
        }
    }

    /// 更新更新时间
    pub fn touch(&mut self) {
        self.update_time = Utc::now();
    }
}

impl Default for BaseMongoFields {
    fn default() -> Self {
        Self::new()
    }
}

/// MongoDB 实体 trait
///
/// 所有 MongoDB 实体都应该实现此 trait
pub trait MongoEntity: Send + Sync {
    /// 获取集合名称
    fn collection_name() -> &'static str;

    /// 获取 ID
    fn id(&self) -> &ObjectId;

    /// 获取创建时间
    fn create_time(&self) -> &DateTime<Utc>;

    /// 获取更新时间
    fn update_time(&self) -> &DateTime<Utc>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id() {
        let id = ObjectId::new();
        let id_str = id.to_string();
        assert_eq!(id_str.len(), 24); // ObjectID 是 24 个十六进制字符

        let parsed = ObjectId::parse(&id_str).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_base_mongo_fields() {
        let mut fields = BaseMongoFields::new();
        let old_update_time = fields.update_time;

        // 等待一小段时间
        std::thread::sleep(std::time::Duration::from_millis(10));

        fields.touch();
        assert!(fields.update_time > old_update_time);
    }
}
