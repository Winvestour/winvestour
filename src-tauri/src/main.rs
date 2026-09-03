// Wocial masaustu kabugu: tek pencere, canli winvestour.com'u app-modu ile acar.
// 19 Tem: tepsi/close-to-tray KALDIRILDI (Tutku "cikis yapamiyorum, tepside
// calismaya devam ediyor") — X = gercek cikis (beklenen davranis, Chrome/VSCode
// gibi). Tek-ornek + pencere boyut/konum hatirlama korundu. Is mantigi yok.
// 27 Tem: "mor ekran logosuz" sikayeti — main penceresi artik visible:false ile
// acilir, ayri "splashscreen" penceresi ilk goruneni olur.
// 30 Tem: "'Uygulamaya git' karti tiklaninca gitmiyor" — remote.json capability
// sadece pencere-kontrol izinleri veriyordu, target="_blank"/window.open() ile
// acilan dis baglantilar (ör. musterinin kendi magaza domaini) hicbir handler
// olmadigi icin sessizce yutuluyordu. WebviewWindowBuilder::on_new_window ile
// yakalanip tauri-plugin-opener'la sistem tarayicisinda aciliyor artik — bunun
// icin "main" penceresi artik tauri.conf.json'da DEKLARATIF degil (builder'a
// sadece INSA ANINDA baglanabiliyor), programatik olarak burada kuruluyor.
// Uygulama-ozel deger (baslik/renk/UA) app kimliginden (identifier) cozuluyor —
// 4 app AYNI ikili degil, her biri kendi configs/*.conf.json'iyla ayri derleniyor,
// bu yuzden identifier her derlemede zaten dogru degeri tasiyor.
//
// 6 Agu — TUTKU'NUN 2 SIKAYETI (ayni kok neden zincirinde):
//   (a) "masaustu uygulamalar bazen ilk acildiginda ekran donuyor takiliyor"
//   (b) "bu uygulamalarin splashleri duz renk, logo falan yok"
// KOK NEDEN: 27 Tem'de eklenen splash UZAK bir Next.js sayfasiydi
// (winvestour.com/desktop-splash) ve main penceresi SABIT 1400 ms sonra
// gosteriliyordu — ikisi de sunucuya bagimli, oysa splash'in tum amaci sunucu
// beklenirken bir sey gostermekti:
//   * Soguk acilista (DNS+TLS+SSR) uzak splash 1400 ms'de bosa cikiyor
//     => kullanici sadece pencerenin backgroundColor'ini (duz renk) goruyor  -> (b)
//   * 1400 ms dolunca main, sayfa yuklensin yuklenmesin gosteriliyor. main
//     `decorations(false)` — baslik cubugunu SITE ciziyor (DesktopTitlebar).
//     Sayfa daha gelmediyse ortaya cikan sey: bos, cercevesiz, kucultulemeyen,
//     KAPATILAMAYAN bir dikdortgen                                            -> (a)
// DUZELTME:
//   1. Splash artik YEREL (desktop/splash/, frontendDist) — ag olmadan bile
//      logo+marka rengi ANINDA cizilir.
//   2. main artik zamanlayiciyla degil, gercekten sayfa yuklendiginde
//      (PageLoadEvent::Finished) gosteriliyor.
//   3. Guvenlik agi: 20 sn icinde hic yukleme bitmezse main YINE gosterilir ama
//      OS baslik cubugu acilarak (set_decorations(true)) — site kendi
//      titlebar'ini cizemedigi durumda pencere en azindan kapatilabilir kalir.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use tauri::{
    webview::PageLoadEvent, window::Color, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_window_state::StateFlags;

/// Yuklemenin bitmesini en fazla bu kadar bekleriz; sonra pencereyi OS baslik
/// cubuguyla yine de goster (kullanici kapatabilsin diye).
const REVEAL_TIMEOUT_MS: u64 = 20_000;
/// NavigationCompleted ile ilk boyama arasindaki kucuk bosluk (beyaz flas).
const PAINT_GRACE_MS: u64 = 250;

struct Brand {
    key: &'static str,
    color: Color,
}

fn brand_for(identifier: &str) -> Brand {
    // Renkler native/assets-*/icon-only.png'lerin arka planiyla BIREBIR ayni
    // (piksel ornekleyerek dogrulandi) — splash logosu sayfaya kusursuz karisiyor.
    if identifier.contains("commerce") {
        Brand { key: "commerce", color: Color(0x00, 0x72, 0xCE, 255) }
    } else if identifier.contains("influencer") {
        Brand { key: "influencer", color: Color(0xE1, 0x1D, 0x48, 255) }
    } else if identifier.contains("reseller") {
        Brand { key: "reseller", color: Color(0x05, 0x96, 0x69, 255) }
    } else if identifier.contains("staff") {
        Brand { key: "staff", color: Color(0x0B, 0x1F, 0x3A, 255) }
    } else if identifier.contains("hub") {
        // 6 Agu — CATI UYGULAMASI. Digerlerinden farki: tek bir urune degil
        // SISTEMIN TAMAMINA acilir (bkz. asagidaki URL dali). Marka rengi
        // Winvestour mavisi.
        Brand { key: "hub", color: Color(0x00, 0x46, 0x8C, 255) }
    } else {
        Brand { key: "social", color: Color(0x7C, 0x3A, 0xED, 255) }
    }
}

/// Ana pencereyi goster + splash'i kapat. Birden fazla kez cagrilabilir; sadece
/// ILK cagri is yapar (hem sayfa-yuklendi olayindan hem guvenlik zamanlayicisindan
/// tetiklendigi icin yaris kacinilmaz).
fn reveal_main(app: &AppHandle, revealed: &Arc<AtomicBool>, needs_os_titlebar: bool) {
    if revealed.swap(true, Ordering::SeqCst) {
        return;
    }
    if let Some(main) = app.get_webview_window("main") {
        if needs_os_titlebar {
            let _ = main.set_decorations(true);
        }
        let _ = main.show();
        let _ = main.set_focus();
        // Pencere gorunur oldu: site preloader'i hala duruyorsa simdi kaldir.
        // (Bkz. PRELOADER_KILL_JS ustundeki not — 2. katman.)
        let _ = main.eval(PRELOADER_KILL_JS);
    }
    if let Some(splash) = app.get_webview_window("splashscreen") {
        let _ = splash.close();
    }
}

/// ⛔ 6 Agu (2. tur) — TUTKU: "bak mesela yine takildi... kapatip acinca 2.de
/// duzeliyor." Semptom: splash KAPANMIS, pencere GORUNUR, ama sitenin kendi
/// `#wv-preloader`'i (mor W + spinner) ekranda ASILI kaliyor.
///
/// Sitedeki gizleme betigi (src/app/layout.tsx) preloader'i `DOMContentLoaded`
/// ya da 6sn'lik `setTimeout` ile kaldiriyor. Ikisi de sayfanin KENDI
/// zamanlamasina bagli: pencere `on_page_load` bitene kadar GIZLI durdugu icin
/// (Chromium gizli sayfalarda zamanlayicilari kisar, DOMContentLoaded ise
/// HTML akisi takilirsa hic tetiklenmez) ilk — soguk — acilista bu iki yol da
/// gecikebiliyor. Ikinci acilista sayfa onbellekten geldigi icin fark edilmiyor.
///
/// COZUM: preloader'in kalkmasini sitenin zamanlayicisina BIRAKMA, kabuk
/// tarafindan garanti et. Iki katman:
///   1) `initialization_script` — her belgede document-start'ta kurulur; sayfa
///      GORUNUR hale gelir gelmez (`visibilitychange`) preloader'i siler.
///      Kisilma bu olayi etkilemez, pencere gosterildigi an tetiklenir.
///   2) `reveal_main` icindeki `eval` — pencereyi gosterdigimiz anda Rust'tan
///      dogrudan calistirilir; 1. katman herhangi bir sebeple kurulamazsa
///      (betik enjeksiyonu basarisiz) son care budur.
///
/// ⚠️ Bu betigi silme: sitedeki gizleyici tek basina YETMIYOR (canli yakalandi).
/// ⛔ DUGUMU SILME, SADECE GIZLE. Ilk surumu `removeChild` yapiyordu; o dugum
/// React agacinin parcasi (SSR HTML'inde var) ve istemcide silinince hydration
/// uyusmazligi olusuyor — canli `ErrorReport`'ta React #418 (24 kayit) ve
/// streaming sirasinda `$RS` icinde "null.parentNode" olarak goruldu.
/// `wv-preloader-out` sinifi gizlemeyi TAMAMEN CSS'te yapar (globals.css:
/// opacity + visibility + pointer-events), DOM yapisi hic degismez.
const PRELOADER_KILL_JS: &str = "(function(){var k=function(){try{var e=document.getElementById('wv-preloader');if(e)e.classList.add('wv-preloader-out');}catch(_){}};var a=function(){k();setTimeout(k,60);setTimeout(k,600);};if(!window.__wvKill){window.__wvKill=1;document.addEventListener('readystatechange',a);document.addEventListener('DOMContentLoaded',a);window.addEventListener('pageshow',a);document.addEventListener('visibilitychange',function(){if(document.visibilityState==='visible')a();});setTimeout(a,3000);}a();})()";

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Ikinci acilis = mevcut pencereyi one getir (yeni pencere acma).
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        // ⛔ 6 Agu — DONMA SIKAYETININ IKINCI (ve daha agir) KOK NEDENI, gercek
        // calistirmayla yakalandi: eklentinin varsayilan bayraklari
        // StateFlags::all() ve icinde VISIBLE var. restore_state() kayitli durum
        // "gorunur" ise pencereyi KENDISI show()+set_focus() ediyor — yani
        // builder'daki `visible(false)` ILK ACILISTAN SONRAKI HER CALISTIRMADA
        // eziliyordu. Sonuc: sayfa daha yuklenmeden, cercevesiz (site kendi
        // baslik cubugunu henuz cizmemis) BOS bir pencere ekrana geliyor ve
        // kapatilamiyordu. DECORATIONS de disarida: asagidaki zaman-asimi
        // dalinda OS cercevesini actigimiz bir calistirma, sonraki acilislara
        // "cerceveli" olarak miras kalmamali.
        // Geriye kalan (SIZE/POSITION/MAXIMIZED/FULLSCREEN) = kullanicinin
        // pencere boyut/konum hatirasi, yani ozelligin asil amaci: korundu.
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    StateFlags::SIZE
                        | StateFlags::POSITION
                        | StateFlags::MAXIMIZED
                        | StateFlags::FULLSCREEN,
                )
                .with_denylist(&["splashscreen"])
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let identifier = app.config().identifier.clone();
            let brand = brand_for(&identifier);
            let app_key = brand.key;
            let title = app.config().product_name.clone().unwrap_or_else(|| "Winvestour".into());
            // Staff app /admin'i acar (herkese acik degil), digerleri kendi ?app=
            // parametresiyle genel siteyi.
            let url: tauri::Url = if app_key == "staff" {
                "https://www.winvestour.com/admin".parse().expect("gecersiz baslangic URL'i")
            } else if app_key == "hub" {
                // ⛔ 6 Agu — CATI UYGULAMASI: `?app=` VERILMEZ. O parametre siteyi
                // tek-urun kabuk moduna sokar (odakli nav, digerleri gizli); catinin
                // amaci tam tersi — "tum sistemin webview olarak dahil oldugu"
                // uygulama. Parametresiz acilinca site normal, tam halinde calisir.
                "https://www.winvestour.com/".parse().expect("gecersiz baslangic URL'i")
            } else {
                format!("https://www.winvestour.com/?app={app_key}").parse().expect("gecersiz baslangic URL'i")
            };
            // ⛔ 6 Agu — CATI UYGULAMASI `WinvestourApp/` ETIKETI GONDERMEZ.
            // Site o etiketi "tek-urun kabuk modu" sinyali olarak kullaniyor ve
            // TANIMADIGI bir deger gorunce (hub) baslik cubugunu Wocial'a
            // dusuruyordu (kurulu kabukta olculdu: pencere basligi Winvestour
            // ama site mor "Wocial" seridi ciziyordu). Catinin amaci zaten
            // sitenin TAM hali → etiketsiz gitmek dogru davranis.
            // `WinvestourDesktop/1.0` KALIYOR: masaustune ozel davranislar
            // (indirme kartlari vb.) ona bakiyor, urun kimligine degil.
            let app_tag = if app_key == "hub" { String::new() } else { format!("WinvestourApp/{app_key} ") };
            let user_agent = format!(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36 {app_tag}WinvestourDesktop/1.0"
            );

            let revealed = Arc::new(AtomicBool::new(false));
            let revealed_load = revealed.clone();

            // Staff paneli daha genis (admin tablolari) — digerleriyle ayni boyut degil.
            let (w, h, min_w, min_h) = if app_key == "staff" { (1360.0, 860.0, 960.0, 640.0) } else { (1280.0, 820.0, 900.0, 600.0) };
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url))
                .title(title)
                .inner_size(w, h)
                .min_inner_size(min_w, min_h)
                .center()
                .decorations(false)
                .visible(false)
                // Sayfa boyanana kadar beyaz yerine marka rengi gorunsun.
                .background_color(brand.color)
                // ⛔ 6 Agu (Tutku: "wommerce'de surukle birak ... calismiyor"):
                // Tauri'nin OS dosya-birakma isleyicisi Windows'ta HTML5
                // drag&drop olaylarini KOMPLE yutuyor (upstream'in kendi
                // dokumani: "This is required to use HTML5 drag and drop APIs
                // on the frontend on Windows"). Site kurucudaki blok/sayfa/
                // kategori/blog siralamalari bu yuzden masaustunde olu
                // duruyordu. Isleyiciyi kullanmiyoruz (uzak sayfaya Tauri API'si
                // acilmiyor) — kapatmanin maliyeti yok, kazanci HTML5 DnD.
                .disable_drag_drop_handler()
                .user_agent(&user_agent)
                // Preloader guvenlik agi — 1. katman (bkz. PRELOADER_KILL_JS).
                .initialization_script(PRELOADER_KILL_JS)
                // Dis baglanti (target="_blank"/window.open) istegi: app icinde YENI
                // pencere ACMA (Deny), sistemin varsayilan tarayicisinda ac.
                .on_new_window(|url, _features| {
                    let _ = tauri_plugin_opener::open_url(url.to_string(), None::<&str>);
                    tauri::webview::NewWindowResponse::Deny
                })
                // Splash'i kapatip main'i gosterme ANI: sabit zamanlayici DEGIL,
                // gercekten sayfanin yuklenmesi. (WebView2'de bu NavigationCompleted;
                // uzak sayfaya IPC ACILMIYOR — bu Rust tarafi bir geri cagirim,
                // guvenlik durusu 27 Tem'deki gibi korunuyor.)
                .on_page_load(move |webview, payload| {
                    if !matches!(payload.event(), PageLoadEvent::Finished) {
                        return;
                    }
                    if revealed_load.load(Ordering::SeqCst) {
                        return; // ilk yukleme disindaki gezinmeler bizi ilgilendirmiyor
                    }
                    // Yuklenen sey bizim sitemiz DEGILSE (ör. WebView2'nin kendi
                    // "sayfaya ulasilamiyor" ekrani) site kendi baslik cubugunu
                    // cizemez => OS cercevesini ac, pencere kapatilabilir kalsin.
                    let is_ours = payload
                        .url()
                        .host_str()
                        .is_some_and(|h| h == "winvestour.com" || h.ends_with(".winvestour.com"));
                    let handle = webview.app_handle().clone();
                    let flag = revealed_load.clone();
                    thread::spawn(move || {
                        thread::sleep(Duration::from_millis(PAINT_GRACE_MS));
                        let inner = handle.clone();
                        let _ = handle.run_on_main_thread(move || {
                            reveal_main(&inner, &flag, !is_ours);
                        });
                    });
                })
                .build()?;

            // Splash: YEREL sayfa (frontendDist = ../splash) — ag beklemez, ilk
            // karede logo + marka rengi cizilir.
            WebviewWindowBuilder::new(
                app,
                "splashscreen",
                WebviewUrl::App(format!("index.html?app={app_key}").into()),
            )
            .title("Winvestour")
            .inner_size(280.0, 320.0)
            .resizable(false)
            .decorations(false)
            .center()
            .always_on_top(true)
            .skip_taskbar(true)
            .shadow(false)
            .background_color(brand.color)
            .build()?;

            // Guvenlik agi: sayfa hic yuklenmezse (ag yok / cok yavas) kullanici
            // splash ekraninda sonsuza dek kalmasin.
            let timeout_handle = app.handle().clone();
            let revealed_timeout = revealed.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(REVEAL_TIMEOUT_MS));
                if revealed_timeout.load(Ordering::SeqCst) {
                    return;
                }
                let inner = timeout_handle.clone();
                let _ = timeout_handle.run_on_main_thread(move || {
                    reveal_main(&inner, &revealed_timeout, true);
                });
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Tauri baslatilamadi");
}
