<div align="center">

<img src="src-tauri/icons/128x128.png" width="110" height="110" alt="Icône Konsol, un K en verre bleu" />

# Konsol

Tous les jeux du PC au même endroit, lancés sans ouvrir leur launcher.

[![KDE Plasma 6](https://img.shields.io/badge/KDE%20Plasma%206-Linux-0b1a4d?style=for-the-badge)](#-installation)

<br />

<img src="docs/hero.png" width="600" alt="Konsol avec Cyberpunk 2077 sélectionné : l'image du jeu en fond, son titre en grand, 0 / 57 succès, le bouton Lancer, et en bas les jaquettes de Cyberpunk, Shipping Store Simulator, Skate 3 Recomp et Isaac" />

</div>

<br />

J'en avais marre d'ouvrir Steam, Heroic ou un launcher différent à chaque fois que je voulais jouer. Alors j'ai fait Konsol : tous les jeux installés sont dans une seule fenêtre, tu choisis, tu lances, et le launcher reste caché derrière.

## 🎮 Tous tes jeux, d'où qu'ils viennent

Konsol va chercher tout seul les jeux Steam, ceux de Heroic (GOG, Epic et Amazon) et les jeux sans launcher qui ont un raccourci dans ton menu, genre un repack dans un préfixe Wine ou un recomp natif. Il garde seulement les vrais jeux : les outils de Steam comme Proton, les logiciels comme Wallpaper Engine et les raccourcis de launchers passent à la trappe.

Le jeu sélectionné prend toute la fenêtre avec son image, et l'interface prend sa couleur. Les flèches `←` et `→` passent d'un jeu à l'autre, `Entrée` ou un double-clic sur une jaquette le lance. Pour les jeux qui sont pas sur Steam, Konsol récupère quand même la jaquette et le fond sur le store Steam, ou sur SteamGridDB si t'as mis ta clé dans les Réglages.

Un jeu qui n'a aucun raccourci ? Clique sur Ajouter un jeu, donne-lui un nom et le chemin complet du fichier qui le lance.

## 🙈 Le launcher reste caché

Quand tu cliques sur Lancer, Konsol démarre Steam en mode silencieux et Heroic sans son interface, du coup c'est directement le jeu qui s'ouvre. Steam doit quand même tourner en fond pour ses jeux, c'est lui qui gère les DRM, mais tu vois pas sa fenêtre.

Au premier lancement d'un jeu depuis Konsol, il repère sa fenêtre et lui crée un raccourci avec la bonne icône. À partir du lancement suivant, ta barre des tâches affiche l'icône du jeu au lieu de l'icône par défaut.

## 🏆 Les succès, avec une notif quand tu en débloques un

Sous le titre du jeu, Konsol affiche où t'en es dans ses succès. Clique dessus pour voir la liste complète avec les icônes et la date de chaque déblocage.

Pendant que tu joues à un jeu lancé depuis Konsol, il surveille tes succès. Dès que t'en débloques un, une notification KDE apparaît avec son icône et sa description. Ça marche pour les jeux Steam et pour les repacks qui enregistrent leurs succès dans `SteamData/user_stats.ini`.

> [!NOTE]
> Pour GOG, les succès viennent du cache de Heroic, donc ils se mettent à jour quand Heroic se synchronise, pas en direct. Pour les repacks, colle une clé API Steam dans les Réglages pour avoir les vrais noms et les icônes, sinon Konsol affiche les noms techniques remis en forme.

## 📥 Installation

Pas encore de version à télécharger, Konsol s'installe depuis les sources pour l'instant. Il te faut Rust, pnpm, `webkit2gtk-4.1`, `kdotool` et `notify-send` (paquet `libnotify`), puis dans le dossier du projet :

```bash
pnpm install
pnpm install-local
```

Konsol apparaît ensuite dans ton menu, avec son icône dans la barre des tâches.

> [!NOTE]
> Konsol est fait pour KDE Plasma 6. Sans `kdotool`, les icônes des jeux dans la barre des tâches ne marchent pas, et sans `notify-send`, pas de notification de succès. Le reste tourne quand même.

## 🔄 Les mises à jour

Pas de mise à jour automatique. Récupère la dernière version du code et relance `pnpm install-local`, ça remplace l'ancienne. Ta config et tes jeux ajoutés à la main ne bougent pas.

## 🔒 Et tes données ?

Konsol lit les fichiers de Steam et de Heroic sur ton PC, il se connecte à aucun de tes comptes.

Ce qui sort de ta machine :

- le nom des jeux sans launcher, envoyé au store Steam pour trouver leur jaquette, puis à SteamGridDB si t'as mis une clé ;
- les numéros de jeux Steam, pour savoir si c'est un jeu ou un logiciel, et pour récupérer les succès des repacks si t'as mis une clé API Steam ;
- les téléchargements des images (jaquettes, fonds, icônes de succès) depuis Steam, GOG et SteamGridDB.

Ce qui est stocké :

- `~/.config/dev.qyrn.konsol/config.json` : tes jeux ajoutés à la main, les jeux masqués et tes clés API (lisible que par toi) ;
- `~/.cache/dev.qyrn.konsol/` : les images téléchargées et les infos trouvées en ligne, tu peux le vider quand tu veux ;
- `~/.local/share/applications/konsol-*.desktop` et `~/.local/share/dev.qyrn.konsol/icons/` : les raccourcis créés pour les icônes de la barre des tâches.

## 🤔 Si ça coince

<details>
<summary><b>Un de mes jeux n'apparaît pas</b></summary>
<br />
Pour un jeu sans launcher, Konsol a besoin d'un raccourci dans ton menu avec la catégorie Jeux, et le fichier qu'il lance doit exister. Les raccourcis qui contiennent « launcher » dans leur nom sont ignorés exprès. Sinon, ajoute-le avec Ajouter un jeu.
</details>

<details>
<summary><b>« Ce jeu n'est plus dans la liste, actualise la bibliothèque. »</b></summary>
<br />
Le jeu a été désinstallé ou déplacé depuis la dernière recherche. Clique sur Actualiser en haut à droite.
</details>

<details>
<summary><b>« Impossible de lancer le jeu »</b></summary>
<br />
La commande du jeu n'a pas pu démarrer, la suite du message dit pourquoi. Pour Steam et Heroic, vérifie qu'ils sont bien installés. Pour un jeu ajouté à la main, vérifie que le chemin est toujours bon.
</details>

<details>
<summary><b>La barre des tâches affiche encore l'icône par défaut</b></summary>
<br />
Konsol apprend la fenêtre du jeu au premier lancement, la bonne icône arrive au lancement suivant. Ça marche seulement si le jeu est lancé depuis Konsol et que <code>kdotool</code> est installé.
</details>

<details>
<summary><b>« Cette clé a un format bizarre, recopie-la sans espace. »</b></summary>
<br />
Les clés SteamGridDB et Steam sont faites uniquement de lettres et de chiffres. Recopie-la depuis le site, sans espace ni guillemet autour.
</details>

<details>
<summary><b>Je reçois pas de notification de succès</b></summary>
<br />
Les notifications arrivent seulement pour un jeu lancé depuis Konsol, et Konsol doit rester ouvert pendant que tu joues. Pour GOG, y'a pas de direct : les succès bougent quand Heroic se synchronise.
</details>

---

<details>
<summary>🛠️ <b>Pour bidouiller le code</b></summary>

<br />

Tauri 2 (Rust 1.99) avec React 19, TypeScript et Vite. Node 26+ et pnpm 12+.

### Prérequis

- Rust et Cargo, pour le backend Tauri
- `webkit2gtk-4.1`, `appmenu-gtk-module`, `libappindicator-gtk3` et `xdotool`, les dépendances système de Tauri sous Linux
- `kdotool` pour retrouver la fenêtre d'un jeu, `notify-send` pour les notifications de succès

### Commandes

```bash
pnpm install
pnpm tauri dev        # lance l'app en mode développement
pnpm test             # tests Rust
pnpm build            # vérifie les types et construit l'interface
pnpm format           # Prettier et cargo fmt
pnpm install-local    # compile la release et l'installe dans ~/.local
cd src-tauri && cargo clippy --all-targets
```

Le logo vient de `design/konsol-icon.svg`. Pour régénérer les icônes, exporte-le en PNG 1024 px avec `magick design/konsol-icon.svg -resize 1024x1024 design/konsol-icon.png`, puis lance `pnpm tauri icon design/konsol-icon.png`.

| Dossier                      | Rôle                                                        |
| ---------------------------- | ----------------------------------------------------------- |
| `src`                        | L'interface React                                           |
| `src-tauri/src/sources`      | La recherche des jeux : Steam, Heroic et raccourcis du menu |
| `src-tauri/src/metadata`     | Les jaquettes et infos trouvées en ligne, gardées en cache  |
| `src-tauri/src/achievements` | La lecture des succès Steam, GOG et repacks                 |
| `src-tauri/src/commands`     | Les commandes appelées par l'interface                      |
| `design`                     | Le logo en SVG                                              |
| `scripts`                    | L'installation locale avec le raccourci du menu             |
| `docs`                       | Les captures du README                                      |

</details>
