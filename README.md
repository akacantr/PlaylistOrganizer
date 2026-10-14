# 🎵 Spotify Playlist Genre Organizer

Spotify çalma listelerinizi analiz edip müzik türlerine göre (**Rock, Pop, Hip-Hop, Türkçe Rap, Türkçe Arabesk, Phonk / Drift & Bass, Metal, Elektronik / Dance, R&B vb.**) sınıflandıran, filtreleyen ve tek tıkla yeni Spotify çalma listelerine ayıran modern full-stack web uygulaması.

---

## 🚀 Öne Çıkan Özellikler

- **🔒 Güvenli Spotify OAuth Girişi**: Spotify Developer App üzerinden kendi hesabınıza güvenle bağlanın.
- **⚡ Çok Katmanlı Akıllı Tür Sınıflandırma Motoru**:
  - **Yerleşik Bilgi Bankası**: Yüzlerce popüler Türkçe (Rap, Arabesk, Rock, Pop, Halk) ve küresel sanatçıyı 0ms gecikmeyle tanır.
  - **Deezer API Entegrasyonu**: Spotify'ın Developer Mode kısıtlamalarını aşarak gerçek zamanlı albüm ve tür analizi yapar.
  - **Kalıcı Disk Önbelleği (`artist_cache.json`)**: Çözümlenen sanatçıları hafızaya alır; listeler sonraki açılışlarda anında yüklenir.
  - **Başlık & Remix Sezgiseli**: Etiketsiz underground/TikTok parçalarını (`phonk`, `montagem`, `automotivo`, `slowed + reverb`, `hardstyle`, `remix` vb.) otomatik sınıflandırır.
- **📊 Görsel Dağılım Grafikleri**: Çalma listenizin tür dağılımını yüzdesel barlarla dinamik olarak sunar.
- **🎯 Esnek Filtreleme & Arama**: Tür etiketlerine tıklayarak listeyi anında daraltın, şarkı veya sanatçı adına göre arayın.
- **🎧 Şarkı Önizleme Oynatıcısı**: Tarayıcı içerisinden şarkıların önizlemelerini doğrudan dinleyin.
- **🪄 Tek Tıkla Ayrıştırma ve Dışa Aktarma**:
  - **Tüm Türleri Otomatik Böl**: Bütün müzik türlerini tek tıkla bağımsız yeni Spotify çalma listelerine dönüştürür.
  - **Seçileni Oluştur**: Yalnızca seçtiğiniz türü veya şarkıları içeren özel bir liste oluşturur.
- **🌙 Spotify Koyu Tema UI**: Glassmorphism efektleri, modern animasyonlar ve responsive tasarım.

---

## 🛠️ Kurulum ve Çalıştırma

### 1. Gereksinimler
- Python 3.9+ yüklü olmalıdır.

### 2. Projeyi Klonlayın ve Bağımlılıkları Yükleyin
```bash
git clone https://github.com/KULLANICI_ADINIZ/PlaylistOrganizer.git
cd PlaylistOrganizer
pip install -r requirements.txt
```

### 3. Spotify Developer Ayarları (2 Dakika)
1. [Spotify Developer Dashboard](https://developer.spotify.com/dashboard) sayfasına gidin ve giriş yapın.
2. **"Create app"** butonuna tıklayın:
   - **App name**: `Playlist Organizer`
   - **App description**: `Çalma listelerini türlerine göre ayıran araç`
   - **Redirect URIs**: Aşağıdaki iki adresi de ekleyip **Add**'e tıklayın:
     - `http://127.0.0.1:8888/callback`
     - `http://localhost:8888/callback`
   - **Which API/SDKs are you planning to use?**: `Web API` seçeneğini işaretleyin.
3. Şartları kabul edip **Save** deyin.
4. Açılan sayfada **Settings** sekmesine gidin.
5. **Client ID** ve **View client secret** diyerek **Client Secret** değerlerini kopyalayın.

### 4. Uygulamayı Başlatın
- Windows için: **`start.bat`** dosyasına çift tıklayın veya terminalden:
```powershell
python run.py
```
- Tarayıcınız otomatik olarak `http://127.0.0.1:8888` adresinde açılacaktır.

### 5. Kullanım
1. Tarayıcıda sağ üstteki **Ayarlar** (çark) simgesine tıklayın.
2. Spotify **Client ID** ve **Client Secret** bilgilerinizi yapıştırıp **Kaydet** deyin.
3. **"Spotify Hesabını Bağla"** butonuna basarak yetkilendirmeyi tamamlayın.
4. İstediğiniz bir çalma listesini seçin ve müzik türlerine göre organize etmeye başlayın!

---

## 📁 Proje Mimarisi

```
PlaylistOrganizer/
├── app/
│   ├── artist_db.py          # Yerleşik sanatçı ve tür bilgi bankası
│   ├── config.py             # Ortam değişkenleri ve yapılandırma yöneticisi
│   ├── genre_classifier.py   # Makro/mikro kural ve sezgisel sınıflandırma motoru
│   ├── main.py               # FastAPI backend rotaları ve API uç noktaları
│   ├── spotify_client.py     # Spotify OAuth, Deezer API ve veri önbelleği
│   ├── static/               # CSS stilleri ve Vanilla JS kontrolcüsü
│   └── templates/            # Responsive Dark Theme HTML arayüzü
├── data/
│   └── artist_cache.json     # 700+ sanatçı için kalıcı tür önbelleği
├── .env.example              # Örnek yapılandırma şablonu
├── .gitignore                # Hassas token ve gizli anahtar koruması
├── README.md                 # Detaylı dokümantasyon
├── requirements.txt          # Python bağımlılıkları
├── run.py                    # Uvicorn sunucu başlatıcı
└── start.bat                 # Tek tıkla Windows başlatıcı
```

---

## 📄 Lisans
Bu proje [MIT Lisansı](LICENSE) ile lisanslanmıştır.
