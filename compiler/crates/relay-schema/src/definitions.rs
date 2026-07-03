/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use common::NamedItem;
use docblock_shared::RELAY_RESOLVER_MODEL_DIRECTIVE_NAME;
use docblock_shared::RELAY_RESOLVER_MODEL_INSTANCE_FIELD;
use docblock_shared::RELAY_RESOLVER_WEAK_OBJECT_DIRECTIVE;
use schema::FieldID;
use schema::Schema;
use schema::Type;

pub trait ResolverType {
    fn is_resolver_object<S: Schema>(&self, schema: &S) -> bool;
    fn is_weak_resolver_object<S: Schema>(&self, schema: &S) -> bool;
    fn is_terse_resolver_object<S: Schema>(&self, schema: &S) -> bool;
}

/// If `type_` is an `@weak` resolver model object, return its single
/// model-instance field (`__relay_model_instance`) — the field a reader uses to
/// read the weak value inline (no DataID pointer). Returns `None` for anything
/// that is not an `@weak` object.
///
/// This is the one place that resolves the weak object's instance field, shared
/// by `field_transform` (routing a weak return to the inline `Composite` arm)
/// and `client_edges` (per-implementor weak classification).
pub fn weak_object_instance_field<S: Schema>(schema: &S, type_: Type) -> Option<FieldID> {
    let Type::Object(object_id) = type_ else {
        return None;
    };
    let object = schema.object(object_id);
    object
        .directives
        .named(*RELAY_RESOLVER_WEAK_OBJECT_DIRECTIVE)?;
    // A weak object is expected to have exactly one field, the magic
    // `__relay_model_instance` field.
    object.fields.first().copied()
}

impl ResolverType for Type {
    fn is_resolver_object<S: Schema>(&self, schema: &S) -> bool {
        if let Type::Object(object_id) = self {
            let object = schema.object(*object_id);
            object
                .directives
                .named(*RELAY_RESOLVER_MODEL_DIRECTIVE_NAME)
                .is_some()
        } else {
            false
        }
    }

    fn is_weak_resolver_object<S: Schema>(&self, schema: &S) -> bool {
        weak_object_instance_field(schema, *self).is_some()
    }

    fn is_terse_resolver_object<S: Schema>(&self, schema: &S) -> bool {
        if let Type::Object(object_id) = self {
            let object = schema.object(*object_id);
            object.fields.iter().any(|field_id| {
                schema.field(*field_id).name.item == *RELAY_RESOLVER_MODEL_INSTANCE_FIELD
            })
        } else {
            false
        }
    }
}
