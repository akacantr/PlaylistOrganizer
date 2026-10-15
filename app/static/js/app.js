const app = {
    state: {
        hasCredentials: false,
        isAuthenticated: false,
        user: null,
        redirectUri: 'http://localhost:8888/callback',
        playlists: [],
        filteredPlaylists: [],
        activePlaylist: null,
        analysis: null,
        genreMode: 'macro',
        selectedGenres: new Set(),
        selectedTrackIds: new Set(),
        trackSearchQuery: '',
        currentAudioUrl: null,
        isPlaying: false,
    },

    elements: {},

    genreColors: [
        '#1DB954', '#3b82f6', '#ec4899', '#8b5cf6', '#f59e0b',
        '#10b981', '#06b6d4', '#f97316', '#6366f1', '#e11d48',
        '#14b8a6', '#84cc16', '#a855f7', '#64748b'
    ],

    init() {
        this.cacheDom();
        this.checkUrlParams();
        this.loadStatus();
        this.setupAudioEvents();
    },

    cacheDom() {
        this.audioPlayer = document.getElementById('global-audio-player');
    },

    setupAudioEvents() {
        if (!this.audioPlayer) return;
        this.audioPlayer.addEventListener('ended', () => {
            this.state.isPlaying = false;
            this.updateAudioUi();
        });
        this.audioPlayer.addEventListener('pause', () => {
            this.state.isPlaying = false;
            this.updateAudioUi();
        });
        this.audioPlayer.addEventListener('play', () => {
            this.state.isPlaying = true;
            this.updateAudioUi();
        });
    },

    checkUrlParams() {
        const params = new URLSearchParams(window.location.search);
        if (params.get('auth') === 'success') {
            this.showToast('Spotify hesabınız başarıyla bağlandı!', 'success');
            window.history.replaceState({}, document.title, window.location.pathname);
        } else if (params.get('auth_error')) {
            this.showToast('Giriş başarısız: ' + params.get('auth_error'), 'error');
            window.history.replaceState({}, document.title, window.location.pathname);
        }
    },

    async loadStatus() {
        try {
            const res = await fetch('/api/status');
            const data = await res.json();

            this.state.hasCredentials = data.has_credentials;
            this.state.isAuthenticated = data.is_authenticated;
            this.state.user = data.user;
            this.state.redirectUri = data.redirect_uri || this.state.redirectUri;

            const uriEl = document.getElementById('settings-redirect-uri');
            if (uriEl) uriEl.textContent = this.state.redirectUri;

            if (this.state.isAuthenticated && this.state.user) {
                this.renderUser(this.state.user);
                this.loadPlaylists();
                this.showPlaylistsView();
            } else {
                this.showAuthView();
            }
        } catch (e) {
            console.error('Error loading status:', e);
            this.showToast('Sunucu durum bilgisi alınamadı.', 'error');
            this.showAuthView();
        }
    },

    renderUser(user) {
        const userPill = document.getElementById('user-pill');
        const userAvatar = document.getElementById('user-avatar');
        const userName = document.getElementById('user-name');
        const logoutBtn = document.getElementById('logout-btn');

        if (user) {
            userPill.classList.remove('hidden');
            userPill.classList.add('flex');
            logoutBtn.classList.remove('hidden');

            userName.textContent = user.display_name || user.id;
            if (user.avatar) {
                userAvatar.src = user.avatar;
                userAvatar.classList.remove('hidden');
            } else {
                userAvatar.src = 'https://ui-avatars.com/api/?name=' + encodeURIComponent(user.display_name) + '&background=1DB954&color=fff';
            }
        } else {
            userPill.classList.add('hidden');
            userPill.classList.remove('flex');
            logoutBtn.classList.add('hidden');
        }
        lucide.createIcons();
    },

    updateNavTabs(activeView) {
        const navTabs = document.getElementById('nav-segmented-tabs');
        if (!navTabs) return;

        if (!this.state.isAuthenticated) {
            navTabs.classList.add('hidden');
            return;
        }

        navTabs.classList.remove('hidden');
        const tabPlaylists = document.getElementById('nav-tab-playlists');
        const tabAnalysis = document.getElementById('nav-tab-analysis');
        const analysisTitle = document.getElementById('nav-analysis-title');

        if (this.state.activePlaylist && this.state.analysis) {
            tabAnalysis.classList.remove('hidden');
            const plName = this.state.activePlaylist.name || 'Analiz';
            if (analysisTitle) {
                analysisTitle.textContent = plName.length > 18 ? plName.slice(0, 16) + '...' : plName;
            }
        } else {
            tabAnalysis.classList.add('hidden');
        }

        if (activeView === 'playlists') {
            tabPlaylists.classList.add('active');
            tabAnalysis.classList.remove('active');
        } else if (activeView === 'analysis') {
            tabAnalysis.classList.add('active');
            tabPlaylists.classList.remove('active');
        }
    },

    showAuthView() {
        this.hideAllViews();
        this.updateNavTabs('auth');
        const viewAuth = document.getElementById('view-auth');
        viewAuth.classList.remove('hidden');

        const neededBox = document.getElementById('auth-credentials-needed');
        const readyBox = document.getElementById('auth-ready');

        if (!this.state.hasCredentials) {
            neededBox.classList.remove('hidden');
            readyBox.classList.add('hidden');
        } else {
            neededBox.classList.add('hidden');
            readyBox.classList.remove('hidden');
        }
        lucide.createIcons();
    },

    showPlaylistsView() {
        this.hideAllViews();
        this.updateNavTabs('playlists');
        document.getElementById('view-playlists').classList.remove('hidden');
        lucide.createIcons();
    },

    showAnalysisView() {
        this.hideAllViews();
        this.updateNavTabs('analysis');
        document.getElementById('view-analysis').classList.remove('hidden');
        lucide.createIcons();
    },

    hideAllViews() {
        document.getElementById('view-auth').classList.add('hidden');
        document.getElementById('view-playlists').classList.add('hidden');
        document.getElementById('view-analysis').classList.add('hidden');
    },

    showHomeView() {
        if (this.state.isAuthenticated) {
            this.showPlaylistsView();
        } else {
            this.showAuthView();
        }
    },

    async loginWithSpotify() {
        try {
            const res = await fetch('/api/auth/login');
            const data = await res.json();
            if (data.auth_url) {
                window.location.href = data.auth_url;
            } else {
                this.showToast('Giriş bağlantısı alınamadı.', 'error');
            }
        } catch (e) {
            this.showToast('Hata oluştu: ' + e.message, 'error');
        }
    },

    async logout() {
        if (!confirm('Spotify oturumunu kapatmak istediğinize emin misiniz?')) return;
        try {
            await fetch('/api/auth/logout', { method: 'POST' });
            this.state.isAuthenticated = false;
            this.state.user = null;
            this.stopPreviewAudio();
            this.renderUser(null);
            this.showAuthView();
            this.showToast('Başarıyla çıkış yapıldı.', 'info');
        } catch (e) {
            this.showToast('Çıkış yapılırken hata oluştu.', 'error');
        }
    },

    async loadPlaylists() {
        const loading = document.getElementById('playlists-loading');
        const grid = document.getElementById('playlists-grid');
        const empty = document.getElementById('playlists-empty');

        loading.classList.remove('hidden');
        grid.innerHTML = '';
        empty.classList.add('hidden');

        try {
            const res = await fetch('/api/playlists');
            if (res.status === 401) {
                this.showAuthView();
                return;
            }
            const data = await res.json();
            this.state.playlists = data.playlists || [];
            this.state.filteredPlaylists = [...this.state.playlists];

            loading.classList.add('hidden');
            this.renderPlaylists();
        } catch (e) {
            loading.classList.add('hidden');
            this.showToast('Çalma listeleri yüklenirken hata oluştu: ' + e.message, 'error');
        }
    },

    filterPlaylists(query) {
        const q = query.trim().toLowerCase();
        if (!q) {
            this.state.filteredPlaylists = [...this.state.playlists];
        } else {
            this.state.filteredPlaylists = this.state.playlists.filter(p =>
                (p.name && p.name.toLowerCase().includes(q)) ||
                (p.owner && p.owner.toLowerCase().includes(q))
            );
        }
        this.renderPlaylists();
    },

    renderPlaylists() {
        const grid = document.getElementById('playlists-grid');
        const empty = document.getElementById('playlists-empty');

        grid.innerHTML = '';
        if (this.state.filteredPlaylists.length === 0) {
            empty.classList.remove('hidden');
            return;
        }

        empty.classList.add('hidden');

        this.state.filteredPlaylists.forEach(pl => {
            const card = document.createElement('div');
            card.className = 'glass-card p-4 rounded-2xl flex flex-col justify-between group cursor-pointer relative';
            card.onclick = () => this.analyzePlaylist(pl.id);

            const imgHtml = pl.image_url
                ? `<img src="${pl.image_url}" alt="${this.escapeHtml(pl.name)}" class="w-full aspect-square object-cover rounded-xl shadow-md mb-3 bg-white/5">`
                : `<div class="w-full aspect-square rounded-xl bg-spotify-gray flex items-center justify-center text-spotify-subdued mb-3"><i data-lucide="music" class="w-12 h-12"></i></div>`;

            const badgeHtml = pl.is_mine
                ? `<span class="inline-flex items-center gap-1 text-[10px] font-bold px-2 py-0.5 rounded-full bg-spotify-green/20 text-spotify-green border border-spotify-green/30">Kendi Listeniz</span>`
                : `<span class="inline-flex items-center gap-1 text-[10px] font-medium px-2 py-0.5 rounded-full bg-white/10 text-spotify-subdued">Takip Edilen</span>`;

            card.innerHTML = `
                <div>
                    <div class="relative overflow-hidden rounded-xl">
                        ${imgHtml}
                        <button class="absolute bottom-5 right-2 w-10 h-10 rounded-full bg-spotify-green hover:bg-spotify-greenHover text-black flex items-center justify-center shadow-xl opacity-0 translate-y-2 group-hover:opacity-100 group-hover:translate-y-0 transition duration-200">
                            <i data-lucide="play" class="w-4 h-4 fill-black"></i>
                        </button>
                    </div>
                    <div class="flex items-center justify-between gap-1 mb-1.5">
                        ${badgeHtml}
                        <span class="text-[11px] text-spotify-subdued">${pl.total_tracks} şarkı</span>
                    </div>
                    <h4 class="font-bold text-sm text-white truncate mb-1" title="${this.escapeHtml(pl.name)}">${this.escapeHtml(pl.name)}</h4>
                    <p class="text-[11px] text-spotify-subdued line-clamp-1 mb-2">Sahibi: ${this.escapeHtml(pl.owner)}</p>
                </div>
                <div class="flex items-center justify-between text-xs text-spotify-subdued pt-2 border-t border-white/5">
                    <span class="text-spotify-green font-semibold text-[11px] group-hover:underline flex items-center gap-1">
                        Türleri Böl <i data-lucide="arrow-right" class="w-3 h-3"></i>
                    </span>
                </div>
            `;
            grid.appendChild(card);
        });

        lucide.createIcons();
    },

    analysisAbortController: null,

    cancelAnalysis() {
        if (this.analysisAbortController) {
            this.analysisAbortController.abort();
            this.analysisAbortController = null;
        }
        const loadingModal = document.getElementById('analysis-loading-modal');
        if (loadingModal) loadingModal.classList.add('hidden');
        this.showToast('Analiz işlemi iptal edildi.', 'info');
    },

    async analyzePlaylist(playlistId) {
        const loadingModal = document.getElementById('analysis-loading-modal');
        const loadingMsg = document.getElementById('analysis-loading-msg');
        loadingModal.classList.remove('hidden');
        loadingMsg.textContent = 'Çalma listesi ve şarkıların tür bilgileri taranıyor...';

        this.analysisAbortController = new AbortController();

        try {
            const res = await fetch(`/api/playlist/${playlistId}/analyze`, {
                signal: this.analysisAbortController.signal
            });
            if (!res.ok) {
                const err = await res.json();
                throw new Error(err.detail || 'Analiz başarısız');
            }
            const data = await res.json();

            this.state.activePlaylist = data.playlist;
            this.state.analysis = data.analysis;

            this.state.selectedTrackIds = new Set(data.analysis.tracks.map(t => t.id));
            this.state.selectedGenres.clear();
            this.state.genreMode = 'macro';
            this.state.trackSearchQuery = '';

            loadingModal.classList.add('hidden');
            this.renderAnalysisView();
            this.showAnalysisView();
        } catch (e) {
            loadingModal.classList.add('hidden');
            if (e.name === 'AbortError') return;
            alert(e.message);
        } finally {
            this.analysisAbortController = null;
        }
    },

    backToPlaylists() {
        this.stopPreviewAudio();
        this.showPlaylistsView();
    },

    renderAnalysisView() {
        const pl = this.state.activePlaylist;
        const analysis = this.state.analysis;

        document.getElementById('active-playlist-title').textContent = pl.name;
        document.getElementById('active-playlist-desc').textContent = pl.description || 'Açıklama yok.';
        document.getElementById('active-playlist-owner').textContent = pl.owner;
        document.getElementById('active-playlist-total').textContent = analysis.total_tracks;
        document.getElementById('active-playlist-spotify-link').href = pl.spotify_url || '#';

        const coverEl = document.getElementById('active-playlist-img');
        if (pl.image_url) {
            coverEl.src = pl.image_url;
            coverEl.classList.remove('hidden');
        } else {
            coverEl.src = 'https://ui-avatars.com/api/?name=' + encodeURIComponent(pl.name) + '&background=242424&color=1DB954';
        }

        document.getElementById('stat-total-tracks').textContent = analysis.total_tracks;
        document.getElementById('stat-categorized-tracks').textContent = analysis.categorized_count;
        document.getElementById('stat-genres-count').textContent = analysis.macro_genres.length;
        document.getElementById('stat-artists-count').textContent = analysis.unique_artists;

        this.renderDistributionBar();

        this.renderGenreChips();

        this.renderTracksTable();

        lucide.createIcons();
    },

    renderDistributionBar() {
        const barContainer = document.getElementById('genre-distribution-bar');
        const label = document.getElementById('genre-distribution-label');
        barContainer.innerHTML = '';

        const genres = this.state.genreMode === 'macro'
            ? this.state.analysis.macro_genres
            : this.state.analysis.micro_genres.slice(0, 15);

        label.textContent = this.state.genreMode === 'macro' ? 'Ana Tür Dağılımı' : 'Detaylı Tür Dağılımı (İlk 15)';

        genres.forEach((g, idx) => {
            if (g.percentage <= 0) return;
            const color = this.genreColors[idx % this.genreColors.length];
            const segment = document.createElement('div');
            segment.className = 'h-full transition-all duration-300 relative group cursor-pointer';
            segment.style.width = `${g.percentage}%`;
            segment.style.backgroundColor = color;
            segment.title = `${g.name}: %${g.percentage} (${g.count} şarkı)`;

            segment.onclick = () => this.toggleGenreFilter(g.name);
            barContainer.appendChild(segment);
        });
    },

    setGenreMode(mode) {
        this.state.genreMode = mode;
        this.state.selectedGenres.clear();

        const btnMacro = document.getElementById('btn-mode-macro');
        const btnMicro = document.getElementById('btn-mode-micro');

        if (mode === 'macro') {
            btnMacro.className = 'px-3 py-1.5 rounded-lg bg-spotify-green text-black transition font-bold';
            btnMicro.className = 'px-3 py-1.5 rounded-lg text-spotify-subdued hover:text-white transition';
        } else {
            btnMicro.className = 'px-3 py-1.5 rounded-lg bg-spotify-green text-black transition font-bold';
            btnMacro.className = 'px-3 py-1.5 rounded-lg text-spotify-subdued hover:text-white transition';
        }

        this.renderDistributionBar();
        this.renderGenreChips();
        this.renderTracksTable();
    },

    renderGenreChips() {
        const container = document.getElementById('genre-chips-container');
        container.innerHTML = '';

        const genres = this.state.genreMode === 'macro'
            ? this.state.analysis.macro_genres
            : this.state.analysis.micro_genres;

        const isAllActive = this.state.selectedGenres.size === 0;
        const allChip = document.createElement('div');
        allChip.className = `genre-pill inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-semibold border ${
            isAllActive
                ? 'bg-spotify-green text-black border-spotify-green'
                : 'bg-spotify-gray/80 text-spotify-subdued border-white/10 hover:text-white'
        }`;
        allChip.innerHTML = `<span>Tümü</span> <span class="text-[10px] opacity-75">(${this.state.analysis.total_tracks})</span>`;
        allChip.onclick = () => {
            this.state.selectedGenres.clear();
            this.renderGenreChips();
            this.renderTracksTable();
        };
        container.appendChild(allChip);

        genres.forEach((g) => {
            const isSelected = this.state.selectedGenres.has(g.name);
            const chip = document.createElement('div');
            chip.className = `genre-pill inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-semibold border ${
                isSelected
                    ? 'bg-spotify-green text-black border-spotify-green'
                    : 'bg-spotify-gray/80 text-white border-white/10 hover:border-spotify-green/40'
            }`;

            chip.innerHTML = `
                <span onclick="app.toggleGenreFilter('${this.escapeHtml(g.name)}', event)">${this.escapeHtml(g.name)}</span>
                <span class="text-[10px] ${isSelected ? 'text-black/80 font-bold' : 'text-spotify-subdued'}">(${g.count})</span>
                <button onclick="app.quickCreateGenrePlaylist('${this.escapeHtml(g.name)}', event)" class="p-0.5 rounded-full hover:bg-black/20 ${isSelected ? 'text-black' : 'text-spotify-green'}" title="'${this.escapeHtml(g.name)}' için Yeni Playlist Yap">
                    <i data-lucide="plus-circle" class="w-3.5 h-3.5"></i>
                </button>
            `;
            container.appendChild(chip);
        });

        lucide.createIcons();
    },

    toggleGenreFilter(genreName, event) {
        if (event) event.stopPropagation();
        if (this.state.selectedGenres.has(genreName)) {
            this.state.selectedGenres.delete(genreName);
        } else {
            this.state.selectedGenres.add(genreName);
        }
        this.renderGenreChips();
        this.renderTracksTable();
    },

    filterTracksByText(query) {
        this.state.trackSearchQuery = query.trim().toLowerCase();
        this.renderTracksTable();
    },

    getFilteredTracks() {
        let tracks = this.state.analysis.tracks;

        if (this.state.selectedGenres.size > 0) {
            tracks = tracks.filter(t => {
                const targetGenres = this.state.genreMode === 'macro' ? t.macro_genres : t.raw_genres;
                return targetGenres.some(g => this.state.selectedGenres.has(g));
            });
        }

        if (this.state.trackSearchQuery) {
            const q = this.state.trackSearchQuery;
            tracks = tracks.filter(t =>
                t.name.toLowerCase().includes(q) ||
                t.artists.some(a => a.toLowerCase().includes(q)) ||
                t.album.toLowerCase().includes(q)
            );
        }

        return tracks;
    },

    renderTracksTable() {
        const tbody = document.getElementById('tracks-table-body');
        const countBadge = document.getElementById('filtered-tracks-count');
        const masterCheckbox = document.getElementById('master-track-checkbox');
        tbody.innerHTML = '';

        const tracks = this.getFilteredTracks();
        countBadge.textContent = `${tracks.length} şarkı`;

        if (tracks.length === 0) {
            tbody.innerHTML = `<tr><td colspan="7" class="text-center py-10 text-spotify-subdued">Filtre kriterlerinize uygun şarkı bulunamadı.</td></tr>`;
            return;
        }

        const allVisibleChecked = tracks.every(t => this.state.selectedTrackIds.has(t.id));
        masterCheckbox.checked = allVisibleChecked;

        tracks.forEach((t, i) => {
            const isChecked = this.state.selectedTrackIds.has(t.id);
            const isAudioPlaying = this.state.currentAudioUrl === t.preview_url && this.state.isPlaying;

            const tr = document.createElement('tr');
            tr.className = `track-row ${!isChecked ? 'excluded' : ''}`;

            const durationStr = this.formatDuration(t.duration_ms);
            const genresToDisplay = (this.state.genreMode === 'macro' ? t.macro_genres : t.raw_genres).slice(0, 3);

            const genreBadgesHtml = genresToDisplay.map(g =>
                `<span onclick="app.toggleGenreFilter('${this.escapeHtml(g)}')" class="inline-block px-2 py-0.5 rounded-full text-[10px] font-medium bg-white/5 hover:bg-spotify-green hover:text-black border border-white/10 text-spotify-subdued transition cursor-pointer mr-1 mb-1">${this.escapeHtml(g)}</span>`
            ).join('');

            const playBtnHtml = t.preview_url
                ? `<button onclick="app.playPreview('${t.preview_url}', '${this.escapeHtml(t.name)}', '${this.escapeHtml(t.artists.join(', '))}', '${t.image_url}')" class="w-7 h-7 rounded-full bg-white/10 hover:bg-spotify-green hover:text-black text-white flex items-center justify-center transition" title="Önizlemeyi Dinle">
                     <i data-lucide="${isAudioPlaying ? 'pause' : 'play'}" class="w-3.5 h-3.5"></i>
                   </button>`
                : `<span class="text-[10px] text-spotify-subdued/50">-</span>`;

            tr.innerHTML = `
                <td class="p-3 text-center">
                    <input type="checkbox" ${isChecked ? 'checked' : ''} onchange="app.toggleTrackSelection('${t.id}', this.checked)" class="w-4 h-4 rounded accent-spotify-green cursor-pointer">
                </td>
                <td class="p-3 text-center text-spotify-subdued">
                    ${playBtnHtml}
                </td>
                <td class="p-3">
                    <div class="flex items-center gap-3">
                        <img src="${t.image_url || ''}" alt="" class="w-9 h-9 rounded object-cover bg-white/10 shrink-0">
                        <div class="max-w-[220px] sm:max-w-xs truncate">
                            <div class="font-bold text-white text-xs truncate" title="${this.escapeHtml(t.name)}">${this.escapeHtml(t.name)}</div>
                            <div class="text-[11px] text-spotify-subdued truncate">${this.escapeHtml(t.artists.join(', '))}</div>
                        </div>
                    </div>
                </td>
                <td class="p-3 text-spotify-subdued hidden md:table-cell max-w-[150px] truncate" title="${this.escapeHtml(t.album)}">
                    ${this.escapeHtml(t.album)}
                </td>
                <td class="p-3 max-w-[200px]">
                    ${genreBadgesHtml || '<span class="text-spotify-subdued text-[11px] italic">Bilinmiyor</span>'}
                </td>
                <td class="p-3 text-right text-spotify-subdued font-mono">
                    ${durationStr}
                </td>
                <td class="p-3 text-center">
                    <a href="${t.spotify_url}" target="_blank" class="text-spotify-subdued hover:text-spotify-green p-1 inline-block transition" title="Spotify'da Aç">
                        <i data-lucide="external-link" class="w-3.5 h-3.5"></i>
                    </a>
                </td>
            `;

            tbody.appendChild(tr);
        });

        this.updateSelectedTracksCounter();
        lucide.createIcons();
    },

    toggleTrackSelection(trackId, isChecked) {
        if (isChecked) {
            this.state.selectedTrackIds.add(trackId);
        } else {
            this.state.selectedTrackIds.delete(trackId);
        }
        this.updateSelectedTracksCounter();
        this.renderTracksTable();
    },

    handleMasterCheckbox(checkbox) {
        const visibleTracks = this.getFilteredTracks();
        visibleTracks.forEach(t => {
            if (checkbox.checked) {
                this.state.selectedTrackIds.add(t.id);
            } else {
                this.state.selectedTrackIds.delete(t.id);
            }
        });
        this.updateSelectedTracksCounter();
        this.renderTracksTable();
    },

    toggleSelectAllTracks() {
        const visibleTracks = this.getFilteredTracks();
        const allChecked = visibleTracks.every(t => this.state.selectedTrackIds.has(t.id));

        visibleTracks.forEach(t => {
            if (allChecked) {
                this.state.selectedTrackIds.delete(t.id);
            } else {
                this.state.selectedTrackIds.add(t.id);
            }
        });

        this.updateSelectedTracksCounter();
        this.renderTracksTable();
    },

    updateSelectedTracksCounter() {
        const badge = document.getElementById('selected-tracks-count');
        if (badge) {
            badge.textContent = this.state.selectedTrackIds.size;
        }
    },

    playPreview(url, title, artist, img) {
        if (!url) {
            this.showToast('Bu parça için Spotify önizlemesi mevcut değil.', 'info');
            return;
        }

        const bar = document.getElementById('audio-preview-bar');
        const imgEl = document.getElementById('audio-preview-img');
        const titleEl = document.getElementById('audio-preview-title');
        const artistEl = document.getElementById('audio-preview-artist');

        if (this.state.currentAudioUrl === url && this.state.isPlaying) {
            this.audioPlayer.pause();
            return;
        }

        this.state.currentAudioUrl = url;
        this.audioPlayer.src = url;
        this.audioPlayer.play().catch(e => console.error(e));

        imgEl.src = img || '';
        titleEl.textContent = title;
        artistEl.textContent = artist;

        bar.classList.remove('hidden');
        this.updateAudioUi();
        this.renderTracksTable();
    },

    togglePreviewAudio() {
        if (!this.audioPlayer.src) return;
        if (this.state.isPlaying) {
            this.audioPlayer.pause();
        } else {
            this.audioPlayer.play().catch(e => console.error(e));
        }
    },

    stopPreviewAudio() {
        if (this.audioPlayer) {
            this.audioPlayer.pause();
            this.audioPlayer.currentTime = 0;
        }
        this.state.isPlaying = false;
        this.state.currentAudioUrl = null;
        const bar = document.getElementById('audio-preview-bar');
        if (bar) bar.classList.add('hidden');
        this.renderTracksTable();
    },

    updateAudioUi() {
        const icon = document.getElementById('audio-play-icon');
        if (icon) {
            icon.setAttribute('data-lucide', this.state.isPlaying ? 'pause' : 'play');
            lucide.createIcons();
        }
    },

    openCreateCustomModal() {
        const selectedUris = this.getSelectedTrackUris();
        if (selectedUris.length === 0) {
            this.showToast('Lütfen önce en az bir şarkı seçin!', 'warning');
            return;
        }

        const plNameInput = document.getElementById('custom-pl-name');
        const countSpan = document.getElementById('custom-modal-track-count');

        let defaultName = this.state.activePlaylist.name;
        if (this.state.selectedGenres.size > 0) {
            const genresArr = Array.from(this.state.selectedGenres).slice(0, 2);
            defaultName += ` - ${genresArr.join(' & ')}`;
        } else {
            defaultName += ` - Seçilenler`;
        }

        plNameInput.value = defaultName;
        countSpan.textContent = selectedUris.length;

        document.getElementById('modal-create-custom').classList.remove('hidden');
        lucide.createIcons();
    },

    closeCreateCustomModal() {
        document.getElementById('modal-create-custom').classList.add('hidden');
    },

    getSelectedTrackUris() {
        const tracks = this.state.analysis.tracks;
        return tracks
            .filter(t => this.state.selectedTrackIds.has(t.id) && t.uri)
            .map(t => t.uri);
    },

    async submitCreateCustomPlaylist() {
        const name = document.getElementById('custom-pl-name').value.trim();
        const desc = document.getElementById('custom-pl-desc').value.trim();
        const isPublic = document.getElementById('custom-pl-public').checked;
        const uris = this.getSelectedTrackUris();

        if (!name) {
            this.showToast('Lütfen bir çalma listesi adı girin.', 'warning');
            return;
        }

        const btn = document.getElementById('btn-submit-create-custom');
        btn.disabled = true;
        btn.innerHTML = `<i data-lucide="loader-2" class="w-4 h-4 animate-spin"></i> Oluşturuluyor...`;
        lucide.createIcons();

        try {
            const res = await fetch('/api/playlist/create', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: name,
                    description: desc,
                    track_uris: uris,
                    is_public: isPublic
                })
            });

            if (!res.ok) {
                const err = await res.json();
                throw new Error(err.detail || 'Oluşturulamadı');
            }

            const data = await res.json();
            this.closeCreateCustomModal();
            this.showSuccessModal('Playlist Başarıyla Oluşturuldu!', [data.playlist]);
        } catch (e) {
            this.showToast('Hata: ' + e.message, 'error');
        } finally {
            btn.disabled = false;
            btn.innerHTML = `<i data-lucide="check" class="w-4 h-4"></i> Listeyi Oluştur`;
            lucide.createIcons();
        }
    },

    async quickCreateGenrePlaylist(genreName, event) {
        if (event) event.stopPropagation();

        const tracks = this.state.analysis.tracks.filter(t => {
            const targetGenres = this.state.genreMode === 'macro' ? t.macro_genres : t.raw_genres;
            return targetGenres.includes(genreName);
        });

        const uris = tracks.map(t => t.uri).filter(u => u);
        if (uris.length === 0) {
            this.showToast('Bu tür için şarkı bulunamadı.', 'warning');
            return;
        }

        const defaultName = `${this.state.activePlaylist.name} - ${genreName}`;
        if (!confirm(`'${genreName}' türündeki ${uris.length} şarkı için '${defaultName}' adında yeni bir Spotify listesi oluşturulsun mu?`)) {
            return;
        }

        try {
            this.showToast('Playlist oluşturuluyor...', 'info');
            const res = await fetch('/api/playlist/create', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    name: defaultName,
                    description: `${this.state.activePlaylist.name} listesinden ${genreName} şarkıları. (Playlist Organizer)`,
                    track_uris: uris,
                    is_public: false
                })
            });

            if (!res.ok) throw new Error('Oluşturulamadı');
            const data = await res.json();
            this.showSuccessModal(`'${genreName}' Listesi Oluşturuldu!`, [data.playlist]);
        } catch (e) {
            this.showToast('Hata: ' + e.message, 'error');
        }
    },

    openSplitAllModal() {
        const modal = document.getElementById('modal-split-all');
        const listContainer = document.getElementById('split-all-genres-list');
        listContainer.innerHTML = '';

        const genres = this.state.genreMode === 'macro'
            ? this.state.analysis.macro_genres
            : this.state.analysis.micro_genres.filter(g => g.count >= 2);

        genres.forEach(g => {
            const row = document.createElement('label');
            row.className = 'flex items-center justify-between p-3 rounded-2xl bg-black/40 hover:bg-black/60 border border-white/5 cursor-pointer transition';

            row.innerHTML = `
                <div class="flex items-center gap-3">
                    <input type="checkbox" checked value="${this.escapeHtml(g.name)}" class="split-genre-checkbox w-4 h-4 rounded accent-spotify-green" onchange="app.updateSplitAllCounter()">
                    <div>
                        <div class="font-bold text-white">${this.escapeHtml(g.name)}</div>
                        <div class="text-[11px] text-spotify-subdued">Yeni liste: "${this.escapeHtml(this.state.activePlaylist.name)} - ${this.escapeHtml(g.name)}"</div>
                    </div>
                </div>
                <div class="text-xs font-semibold text-spotify-green bg-spotify-green/10 px-2.5 py-1 rounded-full border border-spotify-green/20">
                    ${g.count} şarkı
                </div>
            `;
            listContainer.appendChild(row);
        });

        this.updateSplitAllCounter();
        modal.classList.remove('hidden');
        lucide.createIcons();
    },

    closeSplitAllModal() {
        document.getElementById('modal-split-all').classList.add('hidden');
    },

    updateSplitAllCounter() {
        const checkboxes = document.querySelectorAll('.split-genre-checkbox:checked');
        const badge = document.getElementById('split-all-count-badge');
        if (badge) badge.textContent = checkboxes.length;
    },

    async submitSplitAll() {
        const checkedBoxes = Array.from(document.querySelectorAll('.split-genre-checkbox:checked'));
        if (checkedBoxes.length === 0) {
            this.showToast('Lütfen oluşturulacak en az bir tür seçin.', 'warning');
            return;
        }

        const selectedGenreNames = checkedBoxes.map(cb => cb.value);

        const genreData = {};
        selectedGenreNames.forEach(name => {
            genreData[name] = [];
        });

        this.state.analysis.tracks.forEach(track => {
            const trackGenres = this.state.genreMode === 'macro' ? track.macro_genres : track.raw_genres;
            trackGenres.forEach(tg => {
                if (genreData[tg] && track.uri) {
                    if (!genreData[tg].includes(track.uri)) {
                        genreData[tg].push(track.uri);
                    }
                }
            });
        });

        const btn = document.getElementById('btn-submit-split-all');
        btn.disabled = true;
        btn.innerHTML = `<i data-lucide="loader-2" class="w-4 h-4 animate-spin"></i> Oluşturuluyor...`;
        lucide.createIcons();

        try {
            const res = await fetch('/api/playlist/split-all', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    playlist_name: this.state.activePlaylist.name,
                    genre_mode: this.state.genreMode,
                    min_tracks: 1,
                    is_public: false,
                    genre_data: genreData
                })
            });

            if (!res.ok) {
                const err = await res.json();
                throw new Error(err.detail || 'İşlem başarısız');
            }

            const data = await res.json();
            this.closeSplitAllModal();
            this.showSuccessModal(`${data.created_count} Çalma Listesi Başarıyla Oluşturuldu!`, data.playlists);
        } catch (e) {
            this.showToast('Hata: ' + e.message, 'error');
        } finally {
            btn.disabled = false;
            btn.innerHTML = `<i data-lucide="sparkles" class="w-4 h-4"></i> Listeleri Oluştur`;
            lucide.createIcons();
        }
    },

    showSuccessModal(title, playlists) {
        const modal = document.getElementById('modal-success');
        document.getElementById('success-modal-title').textContent = title;

        const list = document.getElementById('success-modal-playlists');
        list.innerHTML = '';

        playlists.forEach(p => {
            const card = document.createElement('div');
            card.className = 'flex items-center justify-between p-3 rounded-2xl bg-black/40 border border-white/5 hover:border-spotify-green/30 transition';
            card.innerHTML = `
                <div>
                    <div class="font-bold text-white text-xs">${this.escapeHtml(p.name)}</div>
                    <div class="text-[11px] text-spotify-subdued">${p.total_tracks} şarkı eklendi</div>
                </div>
                <a href="${p.spotify_url}" target="_blank" class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full bg-spotify-green hover:bg-spotify-greenHover text-black font-bold text-xs transition">
                    Spotify'da Aç <i data-lucide="external-link" class="w-3 h-3"></i>
                </a>
            `;
            list.appendChild(card);
        });

        modal.classList.remove('hidden');
        lucide.createIcons();
    },

    closeSuccessModal() {
        document.getElementById('modal-success').classList.add('hidden');
    },

    openSettingsModal() {
        const modal = document.getElementById('modal-settings');
        const uriInput = document.getElementById('cfg-redirect-uri');
        if (uriInput && this.state.redirectUri) {
            uriInput.value = this.state.redirectUri;
        }
        modal.classList.remove('hidden');
        lucide.createIcons();
    },

    closeSettingsModal() {
        document.getElementById('modal-settings').classList.add('hidden');
    },

    copyText(text) {
        navigator.clipboard.writeText(text).then(() => {
            this.showToast('Adres panoya kopyalandı: ' + text, 'success');
        }).catch(() => {
            this.showToast('Kopyalama başarısız oldu.', 'error');
        });
    },

    copyRedirectUri() {
        this.copyText(this.state.redirectUri);
    },

    async saveSettings() {
        const clientId = document.getElementById('cfg-client-id').value.trim();
        const clientSecret = document.getElementById('cfg-client-secret').value.trim();
        const redirectUriInput = document.getElementById('cfg-redirect-uri');
        const redirectUri = redirectUriInput ? redirectUriInput.value.trim() : this.state.redirectUri;

        if (!clientId || !clientSecret) {
            this.showToast('Lütfen hem Client ID hem de Client Secret girin.', 'warning');
            return;
        }

        try {
            const res = await fetch('/api/config', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    client_id: clientId,
                    client_secret: clientSecret,
                    redirect_uri: redirectUri || 'http://localhost:8888/callback'
                })
            });

            if (!res.ok) throw new Error('Ayarlar kaydedilemedi');

            this.showToast('Ayarlar kaydedildi!', 'success');
            this.closeSettingsModal();
            this.loadStatus();
        } catch (e) {
            this.showToast('Hata: ' + e.message, 'error');
        }
    },

    showToast(message, type = 'info') {
        const container = document.getElementById('toast-container');
        const toast = document.createElement('div');

        let bg = 'bg-spotify-gray text-white border-white/10';
        let icon = 'info';

        if (type === 'success') {
            bg = 'bg-[#122e1b] text-green-300 border-green-500/30';
            icon = 'check-circle';
        } else if (type === 'error') {
            bg = 'bg-[#3b1219] text-red-300 border-red-500/30';
            icon = 'alert-triangle';
        } else if (type === 'warning') {
            bg = 'bg-[#36270b] text-amber-300 border-amber-500/30';
            icon = 'alert-circle';
        }

        toast.className = `pointer-events-auto flex items-center gap-2.5 px-4 py-3 rounded-2xl border shadow-2xl text-xs font-semibold transform transition-all duration-300 translate-y-2 opacity-0 ${bg}`;
        toast.innerHTML = `
            <i data-lucide="${icon}" class="w-4 h-4 shrink-0"></i>
            <span>${this.escapeHtml(message)}</span>
        `;

        container.appendChild(toast);
        lucide.createIcons();

        requestAnimationFrame(() => {
            toast.classList.remove('translate-y-2', 'opacity-0');
        });

        setTimeout(() => {
            toast.classList.add('opacity-0', 'translate-y-2');
            setTimeout(() => toast.remove(), 300);
        }, 4000);
    },

    formatDuration(ms) {
        if (!ms) return '0:00';
        const minutes = Math.floor(ms / 60000);
        const seconds = Math.floor((ms % 60000) / 1000);
        return `${minutes}:${seconds < 10 ? '0' : ''}${seconds}`;
    },

    escapeHtml(str) {
        if (!str) return '';
        return str
            .replace(/&/g, '&amp;')
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#039;');
    }
};

document.addEventListener('DOMContentLoaded', () => {
    app.init();
});
