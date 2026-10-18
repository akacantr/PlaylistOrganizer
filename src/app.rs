use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, Margin, Rounding, ScrollArea, Stroke, Vec2};

use crate::artist_db;
use crate::audio::AudioPlayer;
use crate::cache::ArtistCache;
use crate::classifier::{self, PlaylistAnalysis};
use crate::config::{self, AppConfig};
use crate::spotify::{
    CreatedPlaylistResult, PlaylistSummary, SpotifyAuth, SpotifyClient, UserProfile,
};
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Playlists,
    Analysis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenreMode {
    Macro,
    Micro,
}

#[derive(Debug, Clone)]
pub enum ToastKind {
    Success,
    Error,
    Info,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub created_at: Instant,
    pub duration: Duration,
}

#[allow(dead_code)]
pub enum AppMessage {
    AuthSuccess(String),
    AuthError(String),
    UserProfileLoaded(UserProfile),
    PlaylistsLoaded(Vec<PlaylistSummary>),
    PlaylistAnalysisLoaded(PlaylistSummary, PlaylistAnalysis),
    PlaylistCreated(CreatedPlaylistResult),
    SplitAllProgress(usize, usize),
    SplitAllFinished(Vec<CreatedPlaylistResult>, Vec<String>),
    ActionError(String),
    Toast(String, ToastKind),
}

pub struct PlaylistOrganizerApp {
    config: AppConfig,
    auth: Arc<SpotifyAuth>,
    client: Arc<SpotifyClient>,
    cache: ArtistCache,
    audio: AudioPlayer,

    // Navigation and state
    active_view: ActiveView,
    current_user: Option<UserProfile>,
    playlists: Vec<PlaylistSummary>,
    playlist_search: String,
    active_playlist: Option<PlaylistSummary>,
    analysis: Option<PlaylistAnalysis>,

    // Filtering & selection in Analysis
    genre_mode: GenreMode,
    selected_genres: HashSet<String>,
    selected_track_ids: HashSet<String>,
    track_search: String,

    // UI modals & dialogs
    show_settings_modal: bool,
    settings_client_id: String,
    settings_client_secret: String,
    settings_redirect_uri: String,
    show_secret_text: bool,
    is_authenticating: bool,

    // Split All modal
    show_split_modal: bool,
    split_min_tracks: usize,
    split_is_public: bool,
    is_splitting: bool,
    split_progress: (usize, usize),

    // Create custom modal
    show_custom_modal: bool,
    custom_pl_name: String,
    custom_pl_desc: String,
    custom_is_public: bool,
    is_creating_playlist: bool,

    // Busy loading indicators
    is_loading_playlists: bool,
    is_analyzing: bool,
    analyzing_playlist_name: String,

    // Toasts & async communication
    toasts: Vec<Toast>,
    tx: Sender<AppMessage>,
    rx: Receiver<AppMessage>,
    egui_ctx: egui::Context,
}

impl PlaylistOrganizerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ui::configure_visuals(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let cfg = config::load_config();
        let auth = Arc::new(SpotifyAuth::new());
        let client = Arc::new(SpotifyClient::new());
        let cache = ArtistCache::new(config::get_artist_cache_path());
        let audio = AudioPlayer::new();

        let (tx, rx) = channel();

        let mut app = Self {
            settings_client_id: cfg.client_id.clone(),
            settings_client_secret: cfg.client_secret.clone(),
            settings_redirect_uri: cfg.redirect_uri.clone(),
            config: cfg,
            auth,
            client,
            cache,
            audio,

            active_view: ActiveView::Playlists,
            current_user: None,
            playlists: Vec::new(),
            playlist_search: String::new(),
            active_playlist: None,
            analysis: None,

            genre_mode: GenreMode::Macro,
            selected_genres: HashSet::new(),
            selected_track_ids: HashSet::new(),
            track_search: String::new(),

            show_settings_modal: false,
            show_secret_text: false,
            is_authenticating: false,

            show_split_modal: false,
            split_min_tracks: 1,
            split_is_public: false,
            is_splitting: false,
            split_progress: (0, 0),

            show_custom_modal: false,
            custom_pl_name: String::new(),
            custom_pl_desc: String::new(),
            custom_is_public: false,
            is_creating_playlist: false,

            is_loading_playlists: false,
            is_analyzing: false,
            analyzing_playlist_name: String::new(),

            toasts: Vec::new(),
            tx,
            rx,
            egui_ctx: cc.egui_ctx.clone(),
        };

        // If credentials exist and we have a cached token, trigger user load
        if config::has_valid_credentials(&app.config) && app.auth.is_authenticated() {
            app.load_user_and_playlists();
        } else if !config::has_valid_credentials(&app.config) {
            app.show_settings_modal = true;
        }

        app
    }

    pub fn add_toast(&mut self, message: &str, kind: ToastKind) {
        self.toasts.push(Toast {
            message: message.to_string(),
            kind,
            created_at: Instant::now(),
            duration: Duration::from_secs(4),
        });
    }

    fn load_user_and_playlists(&mut self) {
        self.is_loading_playlists = true;
        let auth = self.auth.clone();
        let client = self.client.clone();
        let cfg = self.config.clone();
        let tx = self.tx.clone();
        let ctx = self.egui_ctx.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            if let Ok(rt) = rt {
                rt.block_on(async {
                    let token = match auth.get_valid_access_token(&cfg).await {
                        Some(t) => t,
                        None => {
                            let _ = tx.send(AppMessage::ActionError(
                                "Spotify oturumu geçerli değil. Lütfen tekrar giriş yapın.".into(),
                            ));
                            ctx.request_repaint();
                            return;
                        }
                    };

                    match client.get_current_user_profile(&token).await {
                        Ok(user) => {
                            let my_id = user.id.clone();
                            let _ = tx.send(AppMessage::UserProfileLoaded(user));
                            ctx.request_repaint();
                            match client.get_user_playlists(&token, &my_id).await {
                                Ok(pls) => {
                                    let _ = tx.send(AppMessage::PlaylistsLoaded(pls));
                                    ctx.request_repaint();
                                }
                                Err(e) => {
                                    let _ = tx.send(AppMessage::ActionError(e));
                                    ctx.request_repaint();
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(AppMessage::ActionError(e));
                            ctx.request_repaint();
                        }
                    }
                });
            }
        });
    }

    fn analyze_playlist(&mut self, playlist: PlaylistSummary) {
        self.is_analyzing = true;
        self.analyzing_playlist_name = playlist.name.clone();

        let auth = self.auth.clone();
        let client = self.client.clone();
        let cache = self.cache.clone();
        let cfg = self.config.clone();
        let tx = self.tx.clone();
        let ctx = self.egui_ctx.clone();
        let pl_id = playlist.id.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            if let Ok(rt) = rt {
                rt.block_on(async {
                    let token = match auth.get_valid_access_token(&cfg).await {
                        Some(t) => t,
                        None => {
                            let _ =
                                tx.send(AppMessage::ActionError("Spotify oturum hatası.".into()));
                            ctx.request_repaint();
                            return;
                        }
                    };

                    let raw_tracks = match client.get_all_playlist_tracks(&token, &pl_id).await {
                        Ok(t) => t,
                        Err(e) => {
                            let _ = tx.send(AppMessage::ActionError(e));
                            ctx.request_repaint();
                            return;
                        }
                    };

                    let mut artist_map: HashMap<String, String> = HashMap::new();
                    for item in &raw_tracks {
                        if let Some(ref t) = item.track {
                            for a in &t.artists {
                                if !a.id.is_empty() && !a.name.is_empty() {
                                    artist_map.insert(a.id.clone(), a.name.clone());
                                }
                            }
                        }
                    }

                    let artists_genres = client
                        .batch_resolve_artists_genres(
                            &artist_map,
                            &cache,
                            artist_db::get_curated_genres,
                        )
                        .await;

                    let analysis = classifier::analyze_playlist_tracks(
                        &raw_tracks,
                        &artists_genres,
                        artist_db::get_curated_genres,
                    );

                    let _ = tx.send(AppMessage::PlaylistAnalysisLoaded(playlist, analysis));
                    ctx.request_repaint();
                });
            }
        });
    }

    fn start_login_flow(&mut self) {
        if !config::has_valid_credentials(&self.config) {
            self.add_toast(
                "Lütfen önce geçerli Client ID ve Client Secret kaydedin.",
                ToastKind::Warning,
            );
            return;
        }

        self.is_authenticating = true;
        let auth_url = self.auth.generate_auth_url(&self.config);

        match self.auth.listen_for_callback(&self.config.redirect_uri) {
            Ok(rx) => {
                let _ = open::that(&auth_url);
                let auth = self.auth.clone();
                let cfg = self.config.clone();
                let tx = self.tx.clone();
                let ctx = self.egui_ctx.clone();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build();

                    if let Ok(rt) = rt {
                        rt.block_on(async {
                            match rx.await {
                                Ok(Ok(code)) => {
                                    match auth.exchange_code_for_token(&cfg, &code).await {
                                        Ok(token) => {
                                            let _ = tx
                                                .send(AppMessage::AuthSuccess(token.access_token));
                                            ctx.request_repaint();
                                        }
                                        Err(e) => {
                                            let _ = tx.send(AppMessage::AuthError(e));
                                            ctx.request_repaint();
                                        }
                                    }
                                }
                                Ok(Err(e)) => {
                                    let _ = tx.send(AppMessage::AuthError(e));
                                    ctx.request_repaint();
                                }
                                Err(_) => {
                                    let _ = tx.send(AppMessage::AuthError(
                                        "Giriş işlemi iptal edildi veya zaman aşımına uğradı."
                                            .into(),
                                    ));
                                    ctx.request_repaint();
                                }
                            }
                        });
                    }
                });
            }
            Err(e) => {
                self.is_authenticating = false;
                self.add_toast(&e, ToastKind::Error);
            }
        }
    }

    fn start_split_all(&mut self) {
        let analysis = match &self.analysis {
            Some(a) => a,
            None => return,
        };
        let active_pl = match &self.active_playlist {
            Some(p) => p,
            None => return,
        };

        let min_tracks = self.split_min_tracks;
        let is_public = self.split_is_public;
        let pl_base_name = active_pl.name.clone();

        let mut genre_uris: HashMap<String, Vec<String>> = HashMap::new();
        match self.genre_mode {
            GenreMode::Macro => {
                for stat in &analysis.macro_genres {
                    if stat.name != "Diğer / Belirsiz" {
                        let raw_uris: Vec<String> = analysis
                            .tracks
                            .iter()
                            .filter(|t| stat.track_ids.contains(&t.id))
                            .map(|t| t.uri.clone())
                            .collect();
                        let unique = SpotifyClient::deduplicate_track_uris(&raw_uris);
                        if unique.len() >= min_tracks {
                            genre_uris.insert(stat.name.clone(), unique);
                        }
                    }
                }
            }
            GenreMode::Micro => {
                for stat in &analysis.micro_genres {
                    if stat.name != "etiketsiz" {
                        let raw_uris: Vec<String> = analysis
                            .tracks
                            .iter()
                            .filter(|t| stat.track_ids.contains(&t.id))
                            .map(|t| t.uri.clone())
                            .collect();
                        let unique = SpotifyClient::deduplicate_track_uris(&raw_uris);
                        if unique.len() >= min_tracks {
                            genre_uris.insert(stat.name.clone(), unique);
                        }
                    }
                }
            }
        }

        if genre_uris.is_empty() {
            self.add_toast(
                "En az eşik sayısı kadar şarkı içeren tür bulunamadı.",
                ToastKind::Warning,
            );
            return;
        }

        self.is_splitting = true;
        let total = genre_uris.len();
        self.split_progress = (0, total);

        let user_id = self
            .current_user
            .as_ref()
            .map(|u| u.id.clone())
            .unwrap_or_default();
        let auth = self.auth.clone();
        let client = self.client.clone();
        let cfg = self.config.clone();
        let tx = self.tx.clone();
        let ctx = self.egui_ctx.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            if let Ok(rt) = rt {
                rt.block_on(async {
                    let token = match auth.get_valid_access_token(&cfg).await {
                        Some(t) => t,
                        None => {
                            let _ = tx.send(AppMessage::ActionError(
                                "Spotify oturumu geçerli değil.".into(),
                            ));
                            ctx.request_repaint();
                            return;
                        }
                    };

                    let mut created = Vec::new();
                    let mut failed = Vec::new();
                    let mut current = 0;

                    for (genre_name, uris) in genre_uris {
                        let name = format!("{} - {}", pl_base_name, genre_name);
                        let desc = format!(
                            "{} listesinden '{}' türündeki şarkılar. (Playlist Organizer)",
                            pl_base_name, genre_name
                        );

                        match client
                            .create_user_playlist(&token, &user_id, &name, &desc, is_public, &uris)
                            .await
                        {
                            Ok(res) => created.push(res),
                            Err(e) => {
                                eprintln!("Playlist split error for {}: {}", genre_name, e);
                                failed.push(genre_name);
                            }
                        }

                        current += 1;
                        let _ = tx.send(AppMessage::SplitAllProgress(current, total));
                        ctx.request_repaint();
                        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                    }

                    let _ = tx.send(AppMessage::SplitAllFinished(created, failed));
                    ctx.request_repaint();
                });
            }
        });
    }

    fn start_create_custom_playlist(&mut self) {
        let uris: Vec<String> = if let Some(ref analysis) = self.analysis {
            analysis
                .tracks
                .iter()
                .filter(|t| self.selected_track_ids.contains(&t.id))
                .map(|t| t.uri.clone())
                .collect()
        } else {
            Vec::new()
        };

        if uris.is_empty() {
            self.add_toast("Lütfen en az bir şarkı seçin.", ToastKind::Warning);
            return;
        }

        let name = self.custom_pl_name.trim().to_string();
        if name.is_empty() {
            self.add_toast("Lütfen bir çalma listesi adı girin.", ToastKind::Warning);
            return;
        }

        let desc = self.custom_pl_desc.trim().to_string();
        let is_public = self.custom_is_public;
        let user_id = self
            .current_user
            .as_ref()
            .map(|u| u.id.clone())
            .unwrap_or_default();

        self.is_creating_playlist = true;
        let auth = self.auth.clone();
        let client = self.client.clone();
        let cfg = self.config.clone();
        let tx = self.tx.clone();
        let ctx = self.egui_ctx.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            if let Ok(rt) = rt {
                rt.block_on(async {
                    let token = match auth.get_valid_access_token(&cfg).await {
                        Some(t) => t,
                        None => {
                            let _ = tx.send(AppMessage::ActionError(
                                "Spotify oturumu geçerli değil.".into(),
                            ));
                            ctx.request_repaint();
                            return;
                        }
                    };

                    match client
                        .create_user_playlist(&token, &user_id, &name, &desc, is_public, &uris)
                        .await
                    {
                        Ok(res) => {
                            let _ = tx.send(AppMessage::PlaylistCreated(res));
                            ctx.request_repaint();
                        }
                        Err(e) => {
                            let _ = tx.send(AppMessage::ActionError(e));
                            ctx.request_repaint();
                        }
                    }
                });
            }
        });
    }

    fn process_incoming_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                AppMessage::AuthSuccess(_) => {
                    self.is_authenticating = false;
                    self.add_toast("Spotify hesabınız başarıyla bağlandı!", ToastKind::Success);
                    self.load_user_and_playlists();
                }
                AppMessage::AuthError(e) => {
                    self.is_authenticating = false;
                    self.add_toast(&format!("Giriş başarısız: {}", e), ToastKind::Error);
                }
                AppMessage::UserProfileLoaded(user) => {
                    self.current_user = Some(user);
                }
                AppMessage::PlaylistsLoaded(pls) => {
                    self.is_loading_playlists = false;
                    self.playlists = pls;
                }
                AppMessage::PlaylistAnalysisLoaded(summary, analysis) => {
                    self.is_analyzing = false;
                    self.active_playlist = Some(summary);
                    self.analysis = Some(analysis);
                    self.active_view = ActiveView::Analysis;
                    self.selected_genres.clear();
                    self.selected_track_ids.clear();
                    self.track_search.clear();
                    self.add_toast("Çalma listesi analizi tamamlandı!", ToastKind::Success);
                }
                AppMessage::PlaylistCreated(res) => {
                    self.is_creating_playlist = false;
                    self.show_custom_modal = false;
                    self.add_toast(
                        &format!("'{}' oluşturuldu ({} şarkı)!", res.name, res.total_tracks),
                        ToastKind::Success,
                    );
                    self.load_user_and_playlists();
                }
                AppMessage::SplitAllProgress(curr, tot) => {
                    self.split_progress = (curr, tot);
                }
                AppMessage::SplitAllFinished(created, failed) => {
                    self.is_splitting = false;
                    self.show_split_modal = false;
                    let msg = format!(
                        "{} çalma listesi başarıyla oluşturuldu! (Başarısız: {})",
                        created.len(),
                        failed.len()
                    );
                    self.add_toast(&msg, ToastKind::Success);
                    self.load_user_and_playlists();
                }
                AppMessage::ActionError(e) => {
                    self.is_loading_playlists = false;
                    self.is_analyzing = false;
                    self.is_splitting = false;
                    self.is_creating_playlist = false;
                    self.add_toast(&e, ToastKind::Error);
                }
                AppMessage::Toast(msg, kind) => {
                    self.add_toast(&msg, kind);
                }
            }
        }

        self.toasts.retain(|t| t.created_at.elapsed() < t.duration);
    }
}

impl eframe::App for PlaylistOrganizerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_incoming_messages();

        if self.is_loading_playlists
            || self.is_analyzing
            || self.is_splitting
            || self.is_authenticating
            || self.is_creating_playlist
            || self.audio.is_playing()
            || !self.toasts.is_empty()
        {
            ctx.request_repaint_after(Duration::from_millis(50));
        }

        let mut do_logout = false;

        // Top Navigation Bar
        egui::TopBottomPanel::top("top_panel")
            .frame(
                egui::Frame::none()
                    .fill(ui::COLOR_PANEL)
                    .inner_margin(Margin::symmetric(16.0, 12.0)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Logo and Title
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("♫")
                                .color(ui::COLOR_SPOTIFY_GREEN)
                                .size(24.0)
                                .strong(),
                        );
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new("Playlist Organizer")
                                    .color(ui::COLOR_TEXT_PRIMARY)
                                    .size(16.0)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new("Spotify Müzik Türü Düzenleyicisi")
                                    .color(ui::COLOR_TEXT_SECONDARY)
                                    .size(11.0),
                            );
                        });
                    });

                    ui.add_space(24.0);

                    // Nav Segments
                    if self.auth.is_authenticated() {
                        ui.horizontal(|ui| {
                            let pls_selected = self.active_view == ActiveView::Playlists;
                            let pls_btn = if pls_selected {
                                ui::primary_button("Çalma Listeleri")
                            } else {
                                ui::secondary_button("Çalma Listeleri")
                            };
                            if ui.add(pls_btn).clicked() {
                                self.active_view = ActiveView::Playlists;
                            }

                            if self.analysis.is_some() {
                                ui.add_space(8.0);
                                let an_selected = self.active_view == ActiveView::Analysis;
                                let an_label = self
                                    .active_playlist
                                    .as_ref()
                                    .map(|p| {
                                        if p.name.len() > 16 {
                                            format!("{}...", &p.name[..14])
                                        } else {
                                            p.name.clone()
                                        }
                                    })
                                    .unwrap_or_else(|| "Analiz".to_string());

                                let an_btn = if an_selected {
                                    ui::primary_button(&an_label)
                                } else {
                                    ui::secondary_button(&an_label)
                                };
                                if ui.add(an_btn).clicked() {
                                    self.active_view = ActiveView::Analysis;
                                }
                            }
                        });
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Settings Gear Button
                        if ui.add(ui::secondary_button("⚙ Ayarlar")).clicked() {
                            self.show_settings_modal = true;
                        }

                        ui.add_space(8.0);

                        // User profile or login button
                        if let Some(ref user) = self.current_user {
                            if ui.add(ui::danger_button("Çıkış Yap")).clicked() {
                                do_logout = true;
                            }

                            ui.add_space(8.0);

                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&user.display_name)
                                        .color(ui::COLOR_TEXT_PRIMARY)
                                        .size(13.0)
                                        .strong(),
                                );
                            });
                        } else {
                            if ui.add(ui::primary_button("Giriş Yap")).clicked() {
                                self.start_login_flow();
                            }
                        }
                    });
                });
            });

        if do_logout {
            self.auth.logout();
            self.current_user = None;
            self.playlists.clear();
            self.analysis = None;
            self.active_playlist = None;
            self.active_view = ActiveView::Playlists;
            self.add_toast("Çıkış yapıldı.", ToastKind::Info);
        }

        // Bottom Audio Preview Player (if active)
        if let Some(track_info) = self.audio.current_track_info() {
            egui::TopBottomPanel::bottom("audio_preview_bar")
                .frame(
                    egui::Frame::none()
                        .fill(ui::COLOR_PANEL)
                        .inner_margin(Margin::symmetric(16.0, 10.0))
                        .stroke(Stroke::new(1.0_f32, ui::COLOR_BORDER)),
                )
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(&track_info.title)
                                    .color(ui::COLOR_TEXT_PRIMARY)
                                    .size(13.0)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new(&track_info.artist)
                                    .color(ui::COLOR_TEXT_SECONDARY)
                                    .size(11.0),
                            );
                        });

                        ui.add_space(32.0);

                        let is_audio_playing = self.audio.is_playing();
                        let play_btn_text = if is_audio_playing {
                            "⏸ Duraklat"
                        } else {
                            "▶ Oynat"
                        };
                        if ui.add(ui::primary_button(play_btn_text)).clicked() {
                            if is_audio_playing {
                                self.audio.pause();
                            } else if self.audio.is_exhausted() {
                                self.audio.replay();
                            } else {
                                self.audio.toggle();
                            }
                        }

                        ui.add_space(8.0);

                        if ui.add(ui::secondary_button("⏹ Durdur")).clicked() {
                            self.audio.stop();
                        }

                        ui.add_space(32.0);

                        ui.label("Ses:");
                        let mut vol = self.audio.volume();
                        if ui
                            .add(egui::Slider::new(&mut vol, 0.0..=1.0).show_value(false))
                            .changed()
                        {
                            self.audio.set_volume(vol);
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new("30s Önizleme")
                                    .color(ui::COLOR_SPOTIFY_GREEN)
                                    .size(11.0),
                            );
                        });
                    });
                });
        }

        // Central Content Area
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(ui::COLOR_BG)
                    .inner_margin(Margin::same(20.0)),
            )
            .show(ctx, |ui| {
                if !self.auth.is_authenticated() {
                    self.render_unauthenticated_view(ui);
                } else {
                    match self.active_view {
                        ActiveView::Playlists => self.render_playlists_view(ui),
                        ActiveView::Analysis => self.render_analysis_view(ui),
                    }
                }
            });

        // Modals & Dialogs
        self.render_settings_modal(ctx);
        self.render_split_modal(ctx);
        self.render_custom_playlist_modal(ctx);

        // Toast Overlay (Top-Right)
        self.render_toasts(ctx);
    }
}

// Sub-renderers
impl PlaylistOrganizerApp {
    fn render_unauthenticated_view(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.label(
                egui::RichText::new("Hoş Geldiniz!")
                    .color(ui::COLOR_TEXT_PRIMARY)
                    .size(28.0)
                    .strong(),
            );
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("Spotify çalma listelerinizi türlerine göre analiz edin, filtreleyin ve tek tıkla bölün.")
                    .color(ui::COLOR_TEXT_SECONDARY)
                    .size(15.0),
            );

            ui.add_space(30.0);

            if !config::has_valid_credentials(&self.config) {
                ui.label(
                    egui::RichText::new("Başlamak için önce Spotify Developer API anahtarlarınızı girin.")
                        .color(ui::COLOR_TEXT_SECONDARY)
                        .size(13.0),
                );
                ui.add_space(12.0);
                if ui.add(ui::primary_button("⚙ API Ayarlarını Aç")).clicked() {
                    self.show_settings_modal = true;
                }
            } else {
                if self.is_authenticating {
                    ui.spinner();
                    ui.add_space(10.0);
                    ui.label("Tarayıcıda Spotify girişi bekleniyor...");
                    ui.add_space(10.0);
                    if ui.add(ui::secondary_button("İptal")).clicked() {
                        self.auth.cancel_login();
                        self.is_authenticating = false;
                    }
                } else {
                    if ui.add(ui::primary_button("Spotify ile Giriş Yap")).clicked() {
                        self.start_login_flow();
                    }
                }
            }
        });
    }

    fn render_playlists_view(&mut self, ui: &mut egui::Ui) {
        let mut do_reload = false;

        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Çalma Listeleriniz")
                    .color(ui::COLOR_TEXT_PRIMARY)
                    .size(20.0)
                    .strong(),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(ui::secondary_button("🔄 Yenile")).clicked() {
                    do_reload = true;
                }

                ui.add_space(12.0);

                let search_box = egui::TextEdit::singleline(&mut self.playlist_search)
                    .hint_text("Listelerde ara...")
                    .desired_width(200.0);
                ui.add(search_box);
            });
        });

        if do_reload {
            self.load_user_and_playlists();
        }

        ui.add_space(16.0);

        if self.is_loading_playlists {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.spinner();
                ui.add_space(10.0);
                ui.label("Çalma listeleriniz yükleniyor...");
            });
            return;
        }

        if self.is_analyzing {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.spinner();
                ui.add_space(10.0);
                ui.label(format!(
                    "'{}' analiz ediliyor... Şarkılar ve sanatçı türleri çözümleniyor.",
                    self.analyzing_playlist_name
                ));
            });
            return;
        }

        let query = self.playlist_search.to_lowercase().trim().to_string();
        let filtered: Vec<PlaylistSummary> = self
            .playlists
            .iter()
            .filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query))
            .cloned()
            .collect();

        if filtered.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(
                    egui::RichText::new("Eşleşen çalma listesi bulunamadı.")
                        .color(ui::COLOR_TEXT_SECONDARY)
                        .size(14.0),
                );
            });
            return;
        }

        let mut to_analyze = None;

        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical(|ui| {
                for pl in filtered {
                    let mut analyze_clicked = false;

                    egui::Frame::none()
                        .fill(ui::COLOR_CARD)
                        .stroke(Stroke::new(1.0_f32, ui::COLOR_BORDER))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(Margin::same(12.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if !pl.image_url.is_empty() {
                                    ui.add(
                                        egui::Image::new(&pl.image_url)
                                            .fit_to_exact_size(Vec2::new(48.0, 48.0))
                                            .rounding(Rounding::same(4.0)),
                                    );
                                    ui.add_space(10.0);
                                }

                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(&pl.name)
                                                .color(ui::COLOR_TEXT_PRIMARY)
                                                .size(15.0)
                                                .strong(),
                                        );
                                        if pl.is_mine {
                                            ui.add_space(6.0);
                                            ui.label(
                                                egui::RichText::new("Sizin")
                                                    .color(ui::COLOR_SPOTIFY_GREEN)
                                                    .size(10.0)
                                                    .strong(),
                                            );
                                        }
                                    });

                                    ui.add_space(2.0);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} şarkı • Sahibi: {}",
                                            pl.total_tracks, pl.owner
                                        ))
                                        .color(ui::COLOR_TEXT_SECONDARY)
                                        .size(12.0),
                                    );
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.add(ui::primary_button("Analiz Et")).clicked() {
                                            analyze_clicked = true;
                                        }
                                    },
                                );
                            });
                        });

                    ui.add_space(8.0);

                    if analyze_clicked {
                        to_analyze = Some(pl);
                        break;
                    }
                }
            });
        });

        if let Some(pl) = to_analyze {
            self.analyze_playlist(pl);
        }
    }

    fn render_analysis_view(&mut self, ui: &mut egui::Ui) {
        let (analysis, active_pl) = match (&self.analysis, &self.active_playlist) {
            (Some(a), Some(p)) => (a.clone(), p.clone()),
            _ => {
                ui.label("Analiz verisi bulunamadı.");
                return;
            }
        };

        let mut go_back = false;
        let mut open_split = false;
        let mut open_custom = false;

        // Header: Playlist Details & Statistics
        ui.horizontal(|ui| {
            if ui.add(ui::secondary_button("← Listelere Dön")).clicked() {
                go_back = true;
            }

            ui.add_space(16.0);

            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(&active_pl.name)
                        .color(ui::COLOR_TEXT_PRIMARY)
                        .size(20.0)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "{} şarkı • {} benzersiz sanatçı • %{} sınıflandırıldı ({} etiketsiz)",
                        analysis.total_tracks,
                        analysis.unique_artists,
                        if analysis.total_tracks > 0 {
                            ((analysis.categorized_count as f32 / analysis.total_tracks as f32)
                                * 100.0)
                                .round()
                        } else {
                            0.0
                        },
                        analysis.uncategorized_count
                    ))
                    .color(ui::COLOR_TEXT_SECONDARY)
                    .size(12.0),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(ui::primary_button("⚡ Tüm Türleri Böl")).clicked() {
                    open_split = true;
                }

                ui.add_space(8.0);

                let selected_count = self.selected_track_ids.len();
                let custom_btn_text = if selected_count > 0 {
                    format!("+ Seçilenlerden Liste ({} Şarkı)", selected_count)
                } else {
                    "+ Seçilenlerden Liste".to_string()
                };

                if ui.add(ui::secondary_button(&custom_btn_text)).clicked() {
                    open_custom = true;
                }
            });
        });

        if go_back {
            self.active_view = ActiveView::Playlists;
            return;
        }

        if open_split {
            self.show_split_modal = true;
        }

        if open_custom {
            if self.selected_track_ids.is_empty() {
                self.add_toast(
                    "Lütfen önce listeden en az bir şarkı seçin.",
                    ToastKind::Warning,
                );
            } else {
                self.custom_pl_name = format!("{} - Seçilenler", active_pl.name);
                self.custom_pl_desc = format!("{} listesinden seçilen şarkılar.", active_pl.name);
                self.show_custom_modal = true;
            }
        }

        ui.add_space(16.0);

        // Genre Mode Toggles & Pills
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Müzik Türü Filtreleri:")
                    .color(ui::COLOR_TEXT_PRIMARY)
                    .size(13.0)
                    .strong(),
            );

            ui.add_space(12.0);

            if ui
                .selectable_label(
                    self.genre_mode == GenreMode::Macro,
                    "Makro Türler (17 Kategori)",
                )
                .clicked()
            {
                self.genre_mode = GenreMode::Macro;
                self.selected_genres.clear();
            }
            if ui
                .selectable_label(
                    self.genre_mode == GenreMode::Micro,
                    "Mikro Türler (Detaylı)",
                )
                .clicked()
            {
                self.genre_mode = GenreMode::Micro;
                self.selected_genres.clear();
            }

            if !self.selected_genres.is_empty() {
                ui.add_space(12.0);
                if ui.add(ui::secondary_button("Filtreleri Temizle")).clicked() {
                    self.selected_genres.clear();
                }
            }
        });

        ui.add_space(8.0);

        let stats_list = match self.genre_mode {
            GenreMode::Macro => &analysis.macro_genres,
            GenreMode::Micro => &analysis.micro_genres,
        };

        ui.horizontal_wrapped(|ui| {
            for stat in stats_list {
                let is_selected = self.selected_genres.contains(&stat.name);
                let label = format!("{} ({})", stat.name, stat.count);
                if ui::genre_badge(ui, &label, is_selected).clicked() {
                    if is_selected {
                        self.selected_genres.remove(&stat.name);
                    } else {
                        self.selected_genres.insert(stat.name.clone());
                    }
                }
            }
        });

        ui.add_space(16.0);

        let search_term = self.track_search.to_lowercase().trim().to_string();
        let selected_genres = self.selected_genres.clone();
        let mode = self.genre_mode;

        let visible_track_ids: Vec<String> = analysis
            .tracks
            .iter()
            .filter(|track| {
                if !search_term.is_empty() {
                    let name_match = track.name.to_lowercase().contains(&search_term);
                    let artist_match = track
                        .artists
                        .iter()
                        .any(|a| a.to_lowercase().contains(&search_term));
                    let album_match = track.album.to_lowercase().contains(&search_term);
                    if !name_match && !artist_match && !album_match {
                        return false;
                    }
                }
                if !selected_genres.is_empty() {
                    let genre_match = match mode {
                        GenreMode::Macro => track
                            .macro_genres
                            .iter()
                            .any(|g| selected_genres.contains(g)),
                        GenreMode::Micro => track
                            .raw_genres
                            .iter()
                            .any(|g| selected_genres.contains(&g.to_lowercase())),
                    };
                    if !genre_match {
                        return false;
                    }
                }
                true
            })
            .map(|t| t.id.clone())
            .collect();

        // Tracks Table Controls: Search + Select All / Deselect All
        ui.horizontal(|ui| {
            let search_box = egui::TextEdit::singleline(&mut self.track_search)
                .hint_text("Şarkı, sanatçı veya albüm ara...")
                .desired_width(260.0);
            ui.add(search_box);

            ui.add_space(16.0);

            if ui.add(ui::secondary_button("Tümünü Seç")).clicked() {
                for id in &visible_track_ids {
                    self.selected_track_ids.insert(id.clone());
                }
            }

            if ui.add(ui::secondary_button("Seçimi Temizle")).clicked() {
                if !selected_genres.is_empty() || !search_term.is_empty() {
                    for id in &visible_track_ids {
                        self.selected_track_ids.remove(id);
                    }
                } else {
                    self.selected_track_ids.clear();
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!(
                        "Seçilen: {} / {} (Filtrelenen: {})",
                        self.selected_track_ids.len(),
                        analysis.total_tracks,
                        visible_track_ids.len()
                    ))
                    .color(ui::COLOR_TEXT_SECONDARY)
                    .size(12.0),
                );
            });
        });

        ui.add_space(10.0);

        ScrollArea::vertical().show(ui, |ui| {
            ui.vertical(|ui| {
                for track in &analysis.tracks {
                    if !search_term.is_empty() {
                        let name_match = track.name.to_lowercase().contains(&search_term);
                        let artist_match = track
                            .artists
                            .iter()
                            .any(|a| a.to_lowercase().contains(&search_term));
                        let album_match = track.album.to_lowercase().contains(&search_term);
                        if !name_match && !artist_match && !album_match {
                            continue;
                        }
                    }

                    if !selected_genres.is_empty() {
                        let genre_match = match mode {
                            GenreMode::Macro => track
                                .macro_genres
                                .iter()
                                .any(|g| selected_genres.contains(g)),
                            GenreMode::Micro => track
                                .raw_genres
                                .iter()
                                .any(|g| selected_genres.contains(&g.to_lowercase())),
                        };
                        if !genre_match {
                            continue;
                        }
                    }

                    let is_checked = self.selected_track_ids.contains(&track.id);
                    let is_current_audio =
                        self.audio.current_url().as_ref() == track.preview_url.as_ref();
                    let is_audio_playing = is_current_audio && self.audio.is_playing();

                    egui::Frame::none()
                        .fill(if is_audio_playing {
                            Color32::from_rgb(26, 44, 32)
                        } else {
                            ui::COLOR_CARD
                        })
                        .stroke(Stroke::new(
                            1.0_f32,
                            if is_audio_playing {
                                ui::COLOR_SPOTIFY_GREEN
                            } else {
                                ui::COLOR_BORDER
                            },
                        ))
                        .rounding(Rounding::same(6.0))
                        .inner_margin(Margin::symmetric(10.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let mut check = is_checked;
                                if ui.checkbox(&mut check, "").changed() {
                                    if check {
                                        self.selected_track_ids.insert(track.id.clone());
                                    } else {
                                        self.selected_track_ids.remove(&track.id);
                                    }
                                }

                                ui.add_space(4.0);

                                if let Some(ref preview_url) = track.preview_url {
                                    let btn_text = if is_audio_playing { "⏸" } else { "▶" };
                                    if ui.button(btn_text).clicked() {
                                        if is_audio_playing {
                                            self.audio.pause();
                                        } else if is_current_audio && !self.audio.is_exhausted() {
                                            self.audio.resume();
                                        } else {
                                            self.audio.ensure_stream();
                                            self.audio.play(
                                                preview_url,
                                                &track.name,
                                                &track.artists.join(", "),
                                                &track.image_url,
                                            );
                                        }
                                    }
                                } else {
                                    ui.add_enabled(false, egui::Button::new("—"));
                                }

                                ui.add_space(8.0);

                                ui.label(
                                    egui::RichText::new(format!("{}.", track.index))
                                        .color(ui::COLOR_TEXT_SECONDARY)
                                        .size(12.0),
                                );

                                ui.add_space(6.0);

                                if !track.image_url.is_empty() {
                                    ui.add(
                                        egui::Image::new(&track.image_url)
                                            .fit_to_exact_size(Vec2::new(32.0, 32.0))
                                            .rounding(Rounding::same(4.0)),
                                    );
                                    ui.add_space(6.0);
                                }

                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(&track.name)
                                                .color(ui::COLOR_TEXT_PRIMARY)
                                                .size(13.0)
                                                .strong(),
                                        );
                                        if !track.spotify_url.is_empty()
                                            && ui.link("↗").on_hover_text("Spotify'da Aç").clicked()
                                        {
                                            let _ = open::that(&track.spotify_url);
                                        }
                                    });

                                    ui.label(
                                        egui::RichText::new(format!(
                                            "{} • {}",
                                            track.artists.join(", "),
                                            track.album
                                        ))
                                        .color(ui::COLOR_TEXT_SECONDARY)
                                        .size(11.0),
                                    );
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let dur_secs = track.duration_ms / 1000;
                                        let mins = dur_secs / 60;
                                        let secs = dur_secs % 60;
                                        ui.label(
                                            egui::RichText::new(format!("{}:{:02}", mins, secs))
                                                .color(ui::COLOR_TEXT_SECONDARY)
                                                .size(12.0),
                                        );

                                        ui.add_space(16.0);

                                        for mg in &track.macro_genres {
                                            let col = ui::get_genre_color(mg);
                                            ui.label(
                                                egui::RichText::new(mg)
                                                    .color(col)
                                                    .size(10.0)
                                                    .strong(),
                                            );
                                            ui.add_space(4.0);
                                        }
                                    },
                                );
                            });
                        });

                    ui.add_space(4.0);
                }
            });
        });
    }

    fn render_settings_modal(&mut self, ctx: &egui::Context) {
        if !self.show_settings_modal {
            return;
        }

        egui::Window::new("Spotify API Ayarları")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(450.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Spotify Developer Dashboard Bilgileri")
                            .color(ui::COLOR_TEXT_PRIMARY)
                            .size(14.0)
                            .strong(),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new("Uygulamanın çalışabilmesi için Spotify Dashboard'da bir uygulama oluşturmalı ve Client ID ile Client Secret bilgilerini girmelisiniz.")
                            .color(ui::COLOR_TEXT_SECONDARY)
                            .size(12.0),
                    );

                    ui.add_space(12.0);

                    ui.label("Client ID:");
                    ui.add(egui::TextEdit::singleline(&mut self.settings_client_id).desired_width(420.0));

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Client Secret:");
                        if ui.selectable_label(self.show_secret_text, "Göster").clicked() {
                            self.show_secret_text = !self.show_secret_text;
                        }
                    });
                    ui.add(
                        egui::TextEdit::singleline(&mut self.settings_client_secret)
                            .password(!self.show_secret_text)
                            .desired_width(420.0),
                    );

                    ui.add_space(8.0);

                    ui.label("Redirect URI (Varsayılan: http://127.0.0.1:8888/callback):");
                    ui.add(egui::TextEdit::singleline(&mut self.settings_redirect_uri).desired_width(420.0));
                    ui.label(
                        egui::RichText::new("Spotify Dashboard uygulamanızın 'Redirect URIs' listesine bu adresi eklemeyi unutmayın!")
                            .color(ui::COLOR_SPOTIFY_GREEN)
                            .size(11.0),
                    );

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if ui.add(ui::primary_button("Kaydet")).clicked() {
                            let new_cfg = AppConfig {
                                client_id: self.settings_client_id.trim().to_string(),
                                client_secret: self.settings_client_secret.trim().to_string(),
                                redirect_uri: self.settings_redirect_uri.trim().to_string(),
                            };
                            if let Err(e) = config::save_config(&new_cfg) {
                                self.add_toast(&format!("Ayarlar kaydedilemedi: {}", e), ToastKind::Error);
                            } else {
                                self.config = new_cfg;
                                self.add_toast("API ayarları kaydedildi!", ToastKind::Success);
                                self.show_settings_modal = false;
                            }
                        }

                        ui.add_space(8.0);

                        if ui.add(ui::secondary_button("İptal")).clicked() {
                            self.show_settings_modal = false;
                        }

                        if config::has_valid_credentials(&self.config) && !self.auth.is_authenticated() {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.add(ui::primary_button("Giriş Yap")).clicked() {
                                    self.show_settings_modal = false;
                                    self.start_login_flow();
                                }
                            });
                        }
                    });
                });
            });
    }

    fn render_split_modal(&mut self, ctx: &egui::Context) {
        if !self.show_split_modal {
            return;
        }

        egui::Window::new("Tüm Müzik Türlerini Böl")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(420.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Müzik türlerinin her biri için ayrı bir Spotify çalma listesi oluşturulacak.")
                            .color(ui::COLOR_TEXT_SECONDARY)
                            .size(12.0),
                    );

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        ui.label("En az şarkı eşiği:");
                        ui.add(egui::DragValue::new(&mut self.split_min_tracks).range(1..=100));
                    });
                    ui.label(
                        egui::RichText::new("Bu sayıdan daha az şarkısı olan türler için çalma listesi oluşturulmaz.")
                            .color(ui::COLOR_TEXT_SECONDARY)
                            .size(11.0),
                    );

                    ui.add_space(8.0);

                    ui.checkbox(&mut self.split_is_public, "Herkese Açık (Public) Çalma Listesi");

                    ui.add_space(16.0);

                    if self.is_splitting {
                        let (curr, tot) = self.split_progress;
                        ui.label(format!("Oluşturuluyor: {} / {}", curr, tot));
                        ui.add(egui::ProgressBar::new(if tot > 0 { curr as f32 / tot as f32 } else { 0.0 }));
                    } else {
                        ui.horizontal(|ui| {
                            if ui.add(ui::primary_button("Bölmeyi Başlat")).clicked() {
                                self.start_split_all();
                            }

                            ui.add_space(8.0);

                            if ui.add(ui::secondary_button("İptal")).clicked() {
                                self.show_split_modal = false;
                            }
                        });
                    }
                });
            });
    }

    fn render_custom_playlist_modal(&mut self, ctx: &egui::Context) {
        if !self.show_custom_modal {
            return;
        }

        egui::Window::new("Seçilen Şarkılardan Çalma Listesi Oluştur")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_width(420.0);
                ui.vertical(|ui| {
                    ui.label(format!(
                        "Seçilen {} şarkı yeni listeye eklenecek.",
                        self.selected_track_ids.len()
                    ));

                    ui.add_space(10.0);

                    ui.label("Çalma Listesi Adı:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.custom_pl_name).desired_width(400.0),
                    );

                    ui.add_space(8.0);

                    ui.label("Açıklama:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.custom_pl_desc).desired_width(400.0),
                    );

                    ui.add_space(8.0);

                    ui.checkbox(
                        &mut self.custom_is_public,
                        "Herkese Açık (Public) Çalma Listesi",
                    );

                    ui.add_space(16.0);

                    if self.is_creating_playlist {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Liste oluşturuluyor ve şarkılar ekleniyor...");
                        });
                    } else {
                        ui.horizontal(|ui| {
                            if ui.add(ui::primary_button("Oluştur")).clicked() {
                                self.start_create_custom_playlist();
                            }

                            ui.add_space(8.0);

                            if ui.add(ui::secondary_button("İptal")).clicked() {
                                self.show_custom_modal = false;
                            }
                        });
                    }
                });
            });
    }

    fn render_toasts(&mut self, ctx: &egui::Context) {
        if self.toasts.is_empty() {
            return;
        }

        egui::Area::new(egui::Id::new("toast_overlay"))
            .anchor(egui::Align2::RIGHT_TOP, Vec2::new(-20.0, 60.0))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    for toast in &self.toasts {
                        let (bg, stroke) = match toast.kind {
                            ToastKind::Success => {
                                (Color32::from_rgb(26, 44, 32), ui::COLOR_SPOTIFY_GREEN)
                            }
                            ToastKind::Error => (Color32::from_rgb(44, 20, 24), ui::COLOR_DANGER),
                            ToastKind::Warning => (
                                Color32::from_rgb(44, 36, 20),
                                Color32::from_rgb(245, 158, 11),
                            ),
                            ToastKind::Info => (ui::COLOR_CARD, ui::COLOR_BORDER),
                        };

                        egui::Frame::none()
                            .fill(bg)
                            .stroke(Stroke::new(1.0_f32, stroke))
                            .rounding(Rounding::same(8.0))
                            .inner_margin(Margin::symmetric(14.0, 10.0))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(&toast.message)
                                        .color(ui::COLOR_TEXT_PRIMARY)
                                        .size(13.0)
                                        .strong(),
                                );
                            });
                        ui.add_space(6.0);
                    }
                });
            });
    }
}
