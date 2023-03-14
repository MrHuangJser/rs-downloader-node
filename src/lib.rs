use manic::{Downloader, ManicError};
use neon::prelude::*;
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

// Return a global tokio runtime or create one if it doesn't exist.
// Throws a JavaScript exception if the `Runtime` fails to create.
fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
    static RUNTIME: OnceCell<Runtime> = OnceCell::new();

    RUNTIME.get_or_try_init(|| Runtime::new().or_else(|err| cx.throw_error(err.to_string())))
}

fn download(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let (deferred, promise) = cx.promise();
    let url = cx.argument::<JsString>(0)?.value(&mut cx);
    let path = cx.argument::<JsString>(1)?.value(&mut cx);
    let runtime = runtime(&mut cx)?;
    let channel = cx.channel();

    runtime.spawn(async move {
        let result = download_file(&url, &path).await;
        deferred.settle_with(&channel, move |mut cx| match result {
            Ok(_) => Ok(cx.undefined()),
            Err(err) => cx.throw_error(err.to_string()),
        })
    });

    Ok(promise)
}

async fn download_file(url: &str, path: &str) -> Result<(), ManicError> {
    let downloader = Downloader::new(url, 1).await?;
    downloader.download_and_save(path).await?;
    Ok(())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("download", download)?;
    Ok(())
}
