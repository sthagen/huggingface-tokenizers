extern crate tokenizers as tk;

use crate::container::Container;
use neon::prelude::*;

/// Decoder
pub struct Decoder {
    pub decoder: Container<dyn tk::tokenizer::Decoder>,
}

declare_types! {
    pub class JsDecoder for Decoder {
        init(_) {
            // This should not be called from JS
            Ok(Decoder {
                decoder: Container::Empty
            })
        }
    }
}

/// byte_level()
fn byte_level(mut cx: FunctionContext) -> JsResult<JsDecoder> {
    let mut decoder = JsDecoder::new::<_, JsDecoder, _>(&mut cx, vec![])?;
    let guard = cx.lock();
    decoder
        .borrow_mut(&guard)
        .decoder
        .to_owned(Box::new(tk::decoders::byte_level::ByteLevel::default()));
    Ok(decoder)
}

/// wordpiece(prefix: String = "##", cleanup: bool)
fn wordpiece(mut cx: FunctionContext) -> JsResult<JsDecoder> {
    let mut prefix = String::from("##");
    if let Some(args) = cx.argument_opt(0) {
        if args.downcast::<JsUndefined>().is_err() {
            prefix = args.downcast::<JsString>().or_throw(&mut cx)?.value() as String;
        }
    }

    let mut cleanup = true;
    if let Some(args) = cx.argument_opt(1) {
        if args.downcast::<JsUndefined>().is_err() {
            cleanup = args.downcast::<JsBoolean>().or_throw(&mut cx)?.value();
        }
    }

    let mut decoder = JsDecoder::new::<_, JsDecoder, _>(&mut cx, vec![])?;
    let guard = cx.lock();
    decoder
        .borrow_mut(&guard)
        .decoder
        .to_owned(Box::new(tk::decoders::wordpiece::WordPiece::new(
            prefix, cleanup,
        )));
    Ok(decoder)
}

/// metaspace(replacement: String = "_", add_prefix_space: bool = true)
fn metaspace(mut cx: FunctionContext) -> JsResult<JsDecoder> {
    let mut replacement = '▁';
    if let Some(args) = cx.argument_opt(0) {
        if args.downcast::<JsUndefined>().is_err() {
            let rep = args.downcast::<JsString>().or_throw(&mut cx)?.value() as String;
            replacement = rep.chars().nth(0).ok_or_else(|| {
                cx.throw_error::<_, ()>("replacement must be a character")
                    .unwrap_err()
            })?;
        }
    };

    let mut add_prefix_space = true;
    if let Some(args) = cx.argument_opt(1) {
        if args.downcast::<JsUndefined>().is_err() {
            add_prefix_space = args.downcast::<JsBoolean>().or_throw(&mut cx)?.value() as bool;
        }
    }

    let mut decoder = JsDecoder::new::<_, JsDecoder, _>(&mut cx, vec![])?;
    let guard = cx.lock();
    decoder
        .borrow_mut(&guard)
        .decoder
        .to_owned(Box::new(tk::decoders::metaspace::Metaspace::new(
            replacement,
            add_prefix_space,
        )));
    Ok(decoder)
}

/// bpe_decoder(suffix: String = "</w>")
fn bpe_decoder(mut cx: FunctionContext) -> JsResult<JsDecoder> {
    let mut suffix = String::from("</w>");
    if let Some(args) = cx.argument_opt(0) {
        if args.downcast::<JsUndefined>().is_err() {
            suffix = args.downcast::<JsString>().or_throw(&mut cx)?.value() as String;
        }
    }

    let mut decoder = JsDecoder::new::<_, JsDecoder, _>(&mut cx, vec![])?;
    let guard = cx.lock();
    decoder
        .borrow_mut(&guard)
        .decoder
        .to_owned(Box::new(tk::decoders::bpe::BPEDecoder::new(suffix)));
    Ok(decoder)
}

/// Register everything here
pub fn register(m: &mut ModuleContext, prefix: &str) -> NeonResult<()> {
    m.export_function(&format!("{}_ByteLevel", prefix), byte_level)?;
    m.export_function(&format!("{}_WordPiece", prefix), wordpiece)?;
    m.export_function(&format!("{}_Metaspace", prefix), metaspace)?;
    m.export_function(&format!("{}_BPEDecoder", prefix), bpe_decoder)?;
    Ok(())
}
