use openrecall_core::AppError;
use tauri_plugin_dialog::DialogExt;

pub fn show_recovery(app: &tauri::AppHandle, error: &AppError) {
    let message = if error.code == "unsupportedSchema" {
        "أُنشئت هذه المكتبة بإصدار أحدث من OpenRecall. احتفظ بملفات المكتبة، ثم نزّل إصدارًا أحدث من صفحة الإصدارات وأعد فتح التطبيق:\nhttps://github.com/wip-bunker-42/OpenRecall/releases/latest\n\nلن تُستبدل مكتبتك بمكتبة فارغة. إعادة المحاولة تعيد تشغيل التطبيق."
    } else {
        "تعذّر فتح ملفات مكتبتك. تحقق من المساحة المتاحة وإمكانية الوصول إلى مجلد بيانات التطبيق، ثم أعد المحاولة. إذا استمر الخطأ، احتفظ بملفات المكتبة ونسختك الاحتياطية قبل إصلاح الملف أو طلب المساعدة.\n\nلن تُستبدل مكتبتك بمكتبة فارغة. إعادة المحاولة تعيد تشغيل التطبيق."
    };
    let handle = app.clone();
    app.dialog()
        .message(message)
        .title("تعذّر فتح مكتبة OpenRecall")
        .buttons(tauri_plugin_dialog::MessageDialogButtons::OkCancelCustom(
            "إعادة المحاولة".into(),
            "إغلاق".into(),
        ))
        .show(move |retry| {
            if retry {
                handle.request_restart();
            } else {
                handle.exit(0);
            }
        });
}
