extern crate tokenizers as tk;

use crate::utils::Container;
use neon::prelude::*;

/// Model
pub struct Model {
    pub model: Container<dyn tk::tokenizer::Model + Sync>,
}

declare_types! {
    pub class JsModel for Model {
        init(_) {
            // This should not be called from JS
            Ok(Model {
                model: Container::Empty
            })
        }

        /// save(folder: string, name?: string)
        method save(mut cx) {
            let folder = cx.argument::<JsString>(0)?.value();

            let name = if let Some(name_arg) = cx.argument_opt(1) {
                if args.downcast::<JsUndefined>().is_err() {
                    Some(name_arg.downcast_or_throw::<JsString, _>(&mut cx)?.value())
                } else {
                    None
                }
            } else {
                None
            };

            let this = cx.this();
            let guard = cx.lock();
            let result = this.borrow(&guard).model.execute(|model| {
                model.unwrap().save(Path::new(&folder), name.as_deref())
            });

            match result {
                Ok(r) => {
                    let array = JsArray::new(&mut cx, r.len() as u32);
                    for (i, path) in r.into_iter().enumerate() {
                        let n = JsString::new(&mut cx, path.to_string_lossy().into_owned());
                        array.set(&mut cx, i as u32, n)?;
                    }
                    Ok(array.upcast())
                },
                Err(e) => cx.throw_error(format!("{}", e))
            }
        }
    }
}

/// bpe_from_files(vocab: String, merges: String, options?: {
///   cache_capacity?: number,
///   dropout?: number,
///   unk_token?: String,
///   continuing_subword_prefix?: String,
///   end_of_word_suffix?: String
/// })
pub fn bpe_from_files(mut cx: FunctionContext) -> JsResult<JsModel> {
    let vocab = cx.argument::<JsString>(0)?.value() as String;
    let merges = cx.argument::<JsString>(1)?.value() as String;
    let options = cx.argument_opt(2);

    let mut model = JsModel::new::<_, JsModel, _>(&mut cx, vec![])?;
    let mut builder = tk::models::bpe::BPE::from_files(&vocab, &merges)
        .or_else(|e| cx.throw_error(format!("{}", e)))?;

    if let Some(options) = options {
        if let Ok(options) = options.downcast::<JsObject>() {
            if let Ok(cache_capacity) = options.get(&mut cx, "cache_capacity") {
                if let Err(_) = cache_capacity.downcast::<JsUndefined>() {
                    let cache_capacity = cache_capacity
                        .downcast::<JsNumber>()
                        .or_throw(&mut cx)?
                        .value() as usize;
                    builder = builder.cache_capacity(cache_capacity);
                }
            }
            if let Ok(dropout) = options.get(&mut cx, "dropout") {
                if let Err(_) = dropout.downcast::<JsUndefined>() {
                    let dropout = dropout.downcast::<JsNumber>().or_throw(&mut cx)?.value() as f32;
                    builder = builder.dropout(dropout);
                }
            }
            if let Ok(unk_token) = options.get(&mut cx, "unk_token") {
                if let Err(_) = unk_token.downcast::<JsUndefined>() {
                    let unk_token =
                        unk_token.downcast::<JsString>().or_throw(&mut cx)?.value() as String;
                    builder = builder.unk_token(unk_token);
                }
            }
            if let Ok(prefix) = options.get(&mut cx, "continuing_subword_prefix") {
                if let Err(_) = prefix.downcast::<JsUndefined>() {
                    let prefix = prefix.downcast::<JsString>().or_throw(&mut cx)?.value() as String;
                    builder = builder.continuing_subword_prefix(prefix);
                }
            }
            if let Ok(suffix) = options.get(&mut cx, "end_of_word_suffix") {
                if let Err(_) = suffix.downcast::<JsUndefined>() {
                    let suffix = suffix.downcast::<JsString>().or_throw(&mut cx)?.value() as String;
                    builder = builder.end_of_word_suffix(suffix);
                }
            }
        }
    }

    match builder.build() {
        Ok(bpe) => {
            let guard = cx.lock();
            model.borrow_mut(&guard).model.to_owned(Box::new(bpe));
        }
        Err(e) => return cx.throw_error(format!("{}", e)),
    };

    Ok(model)
}

/// bpe_empty()
pub fn bpe_empty(mut cx: FunctionContext) -> JsResult<JsModel> {
    let mut model = JsModel::new::<_, JsModel, _>(&mut cx, vec![])?;
    let bpe = tk::models::bpe::BPE::default();

    let guard = cx.lock();
    model.borrow_mut(&guard).model.to_owned(Box::new(bpe));

    Ok(model)
}

/// wordpiece_from_files(vocab: String, options?: {
///   unkToken?: String = "[UNK]",
///   maxInputCharsPerWord?: number = 100,
/// })
pub fn wordpiece_from_files(mut cx: FunctionContext) -> JsResult<JsModel> {
    let vocab = cx.argument::<JsString>(0)?.value() as String;
    let options = cx.argument_opt(1);

    let mut unk_token = String::from("[UNK]");
    let mut max_input_chars_per_word = Some(100);

    if let Some(options) = options {
        if let Ok(options) = options.downcast::<JsObject>() {
            if let Ok(unk) = options.get(&mut cx, "unkToken") {
                if let Err(_) = unk.downcast::<JsUndefined>() {
                    unk_token = unk.downcast::<JsString>().or_throw(&mut cx)?.value() as String;
                }
            }
            if let Ok(max) = options.get(&mut cx, "maxInputCharsPerWord") {
                if let Err(_) = max.downcast::<JsUndefined>() {
                    max_input_chars_per_word =
                        Some(max.downcast::<JsNumber>().or_throw(&mut cx)?.value() as usize);
                }
            }
        }
    }

    let wordpiece =
        tk::models::wordpiece::WordPiece::from_files(&vocab, unk_token, max_input_chars_per_word)
            .or_else(|e| cx.throw_error(format!("{}", e)))?;

    let mut model = JsModel::new::<_, JsModel, _>(&mut cx, vec![])?;
    let guard = cx.lock();
    model.borrow_mut(&guard).model.to_owned(Box::new(wordpiece));

    Ok(model)
}

/// wordpiece_empty()
pub fn wordpiece_empty(mut cx: FunctionContext) -> JsResult<JsModel> {
    let mut model = JsModel::new::<_, JsModel, _>(&mut cx, vec![])?;
    let wordpiece = tk::models::wordpiece::WordPiece::default();

    let guard = cx.lock();
    model.borrow_mut(&guard).model.to_owned(Box::new(wordpiece));

    Ok(model)
}

/// Register everything here
pub fn register(m: &mut ModuleContext, prefix: &str) -> NeonResult<()> {
    m.export_function(&format!("{}_BPE_from_files", prefix), bpe_from_files)?;
    m.export_function(&format!("{}_BPE_empty", prefix), bpe_empty)?;
    m.export_function(
        &format!("{}_WordPiece_from_files", prefix),
        wordpiece_from_files,
    )?;
    m.export_function(&format!("{}_WordPiece_empty", prefix), wordpiece_empty)?;
    Ok(())
}
