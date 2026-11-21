# Konsol — Direction artistique « Carrousel » (B)

Référence vivante : `KCarrousel.dc.html` (écrans) et `konsol-proto.js` (données, états, couleurs).
Overlays partagés : `KPalette`, `KAdd`, `KAch`, `KSettings`.
Toute amélioration ou tout ajout doit pouvoir se poser à côté des écrans existants sans qu'on voie la couture.

---

## 1. Intention

Un launcher qui ressemble à un **salon de console** : un seul jeu au centre, en grand, éclairé par sa propre couleur. Tout le reste s'efface.

- **Un héros à la fois.** La jaquette sélectionnée est l'élément le plus grand et le plus lumineux de l'écran. Rien ne rivalise avec elle.
- **Le jeu colore l'interface.** Fond, halo, lueur du bouton Lancer et barres de progression prennent l'accent du jeu sélectionné.
- **Symétrie.** L'accueil est centré sur un axe vertical. Les autres écrans (bibliothèque, fiche, réglages) reviennent à un alignement à gauche, mais gardent la même barre du haut.
- **Verre sombre.** Les surfaces interactives sont du verre dépoli sur un fond d'encre, jamais des aplats opaques colorés.
- **Peu de texte, gros texte.** Titres en Bricolage condensé très gras. Le reste en petites capitales ou en gris discret.

---

## 2. Couleurs

### Fond et texte
| Rôle | Valeur |
|---|---|
| Encre (fond racine) | `#05060c` |
| Texte principal | `#f4f6ff` |
| Texte secondaire | `rgba(244,246,255,.62)` à `.72` |
| Texte tertiaire (aides clavier, compteurs) | `rgba(244,246,255,.5)` à `.55` |
| Filets | `rgba(255,255,255,.14)` |
| Panneau modal | `rgba(16,19,32,.94)` |
| Voile derrière un modal | `rgba(5,6,12,.6)` + `blur(8px)` |

### Accent
- **Chaque jeu a sa couleur** (champ `accent` dans `konsol-proto.js`). Elle est choisie à la main, saturée et lisible sur l'encre, et doit rester proche de la couleur dominante de la jaquette.
- L'accent par défaut (bibliothèque vide, actions système) est `#4a7bff`.
- L'accent sert **uniquement** à :
  - la lueur du bouton Lancer : `box-shadow: 0 0 52px -6px <accent>` ;
  - le halo de fond : ellipse floutée `blur(150px)`, opacité `.3` ;
  - la barre de l'onglet actif et les barres de progression (succès), avec un `box-shadow: 0 0 12px <accent>` ;
  - l'ombre de la jaquette sélectionnée : `0 50px 90px -30px <accent>` ;
  - le point « en jeu » qui pulse.
- **Jamais** l'accent comme fond de bouton plein, ni comme couleur de texte de paragraphe.
- Les changements d'accent sont animés (`transition: background .7s`, `box-shadow .7s`).

### Statut
- Succès / connecté : `#5be38a` (petit point de 8 à 10 px uniquement).
- Erreur : `#ff8f80` (texte d'erreur de formulaire).

---

## 3. Verre

La recette standard, à reprendre telle quelle :

```
background: rgba(255,255,255,.07);   /* .08 pour un bouton */
border: 1px solid rgba(255,255,255,.14);   /* .20 pour un bouton */
backdrop-filter: blur(26px) saturate(170%);
```

- Survol d'un bouton en verre : fond `rgba(255,255,255,.16)`.
- Panneaux de contenu (réglages, cartes d'info) : `rgba(10,12,22,.6)` + `blur(26px)`, bordure `rgba(255,255,255,.1)`.
- Le verre n'est beau **que sur une image**. Sur un fond d'encre uni, préférer un filet ou une surface `rgba(255,255,255,.05)`.

---

## 4. Typographie

Une seule famille : **Bricolage Grotesque Variable** (fournie par le design system).

| Usage | Taille | Graisse | Chasse | Interligne |
|---|---|---|---|---|
| Titre du jeu (accueil, centré, une ligne) | 96 / 80 / 64 / 56 px selon la longueur du titre (`t2`) | 800 | `font-stretch: 75%` | .9 |
| Titre de page (Bibliothèque, Réglages) | 72 à 76 px | 800 | 75 % | .9 |
| Chrono « En jeu » | 120 px, `tabular-nums` | 800 | 75 % | 1 |
| Titre de modal | 40 à 42 px | 800 | 75 % | .95 à 1 |
| Titre de rayon (Steam, GOG…) | 30 px | 800 | 75 % | — |
| Grand chiffre (stats) | 42 px | 800 | 75 % | 1 |
| Titre sur une jaquette sans image | 22 à 38 px | 800 | 75 % | .9 |
| Onglets de navigation | 17 px | 700 | normale | — |
| Bouton Lancer | 19 px | 700 | normale | — |
| Bouton secondaire | 16 px | 600 | normale | — |
| Méta (genre, temps de jeu) | 14 à 15 px | 600 | normale | — |
| Étiquette en capitales (source, « En jeu ») | 11 à 14 px, `letter-spacing: .14em`, majuscules | 700 | normale | — |
| Chemin d'exécutable, clé API | 13 à 14 px, `ui-monospace, Consolas` | 400 | — | — |

Règles :
- Le **condensé 75 %** est réservé aux titres et aux grands chiffres. Le texte courant reste en chasse normale.
- Les titres de jeu utilisent l'échelle automatique `t1` / `t2` / `t3`, calculée selon la longueur. On ne fixe jamais une taille en dur pour un titre de jeu.
- Tous les compteurs (succès, temps, positions) sont en `font-variant-numeric: tabular-nums`.
- Sur une image, un titre porte `text-shadow: 0 6px 40px rgba(0,0,0,.5)`.

---

## 5. Formes et espacement

| Élément | Rayon |
|---|---|
| Jaquette du carrousel | 22 px |
| Jaquette en rayon / grille | 16 px |
| Grande jaquette (fiche) | 26 px |
| Modal | 28 à 30 px |
| Panneau de contenu | 24 px |
| Champ de saisie | 14 px |
| Boutons, chips, onglets pilule | 999 px (pilule) |
| Bouton icône | cercle, 42 à 52 px |

- **Marge latérale** des pages : 64 px (fiche : 80 px).
- **Barre du haut** : hauteur ~90 px, `padding: 22px 40px`.
- **Écarts** : 10 à 12 px entre boutons, 16 px entre jaquettes d'un rayon, 26 px entre rayons, 56 à 64 px entre colonnes majeures.
- Format des jaquettes : toujours **2:3** (portrait). Les fonds héros sont en paysage, plein cadre.

---

## 6. Structure des écrans

### Barre du haut (commune à tous les écrans)
Grille à 3 colonnes `1fr auto 1fr` :
- **Gauche** : BrandMark + « Konsol » (26 px, 800, condensé).
- **Centre** : onglets texte *Accueil · Bibliothèque · Réglages*, espacés de 34 px. L'onglet actif est blanc, avec une barre de 3 px de l'accent en dessous (et sa lueur). Les autres sont en texte secondaire.
- **Droite** : indicateur « en jeu » (si une session est active), horloge, pilule *Rechercher · Ctrl K*, bouton rond `+`.

Ne pas ajouter d'onglet sans retirer quelque chose : trois onglets maximum. Un nouvel outil passe par la palette (Ctrl K) ou par les réglages.

### Accueil — le carrousel
- Jaquette centrale **280 × 420**, posée à 116 px du haut.
- Voisines : décalage horizontal de ±236 px pour le rang 1, puis +150 px par rang. Échelle `.76 / .62 / .52`, opacité `1 / .7 / .4 / .16`. Au-delà du rang 3, invisible.
- Jaquette sélectionnée : contour `2px solid rgba(255,255,255,.85)` décalé de 5 px, ombre teintée par l'accent.
- Flèches `‹ ›` rondes en verre (52 px), de part et d'autre.
- Sous le carrousel, centré : chips méta → titre sur une ligne → actions (*Lancer* · *x / y succès* · *Fiche du jeu*).
- Pied centré : position `3 / 14` + aides clavier.

### Accueil — en jeu
Remplace le carrousel. Centré verticalement :
jaquette 240 × 360 avec un halo d'accent qui respire → « ● EN JEU » → chrono 120 px → phrase d'aide → *Arrêter la session*.

### Accueil — bibliothèque vide
Cinq emplacements de jaquette en pointillés aux positions du carrousel (le central porte un `+`), puis le titre « Votre étagère est vide », une phrase d'explication et deux actions (*Analyser ce PC* en bouton principal, *Ajouter un jeu* en secondaire).

### Bibliothèque — rayons
Un rayon horizontal par source (Steam, GOG, Epic, Amazon, Bureau, Manuel), seulement s'il contient au moins un jeu.
Titre du rayon et compteur, puis des jaquettes de 150 × 225 en défilement horizontal, avec une pastille de temps de jeu en haut à droite. Au survol : `scale(1.05)` et ombre teintée.

### Fiche du jeu
Deux colonnes : grande jaquette 400 × 600 à gauche, contenu à droite.
Retour → méta (studio · année · genre) → titre → actions → bandeau de 4 stats séparées par des filets (*Temps de jeu · Succès · Sur le disque · Dernière session*) → ligne exécutable + *Voir les succès*.

### Overlays
La palette, l'ajout de jeu et les succès s'ouvrent **par-dessus** l'écran courant, avec un voile flouté. Le clic sur le voile ou Échap les ferment. Les réglages sont une page, pas un overlay.

---

## 7. Composants

### Bouton Lancer (un seul par écran)
```
padding: 17px 52px; border-radius: 999px;
font-size: 19px; font-weight: 700; color: #fff;
border: 1px solid rgba(255,255,255,.55);
background: linear-gradient(rgba(255,255,255,.4), rgba(255,255,255,.08));
box-shadow: 0 0 52px -6px <accent>, inset 0 1px 0 rgba(255,255,255,.7);
backdrop-filter: blur(26px) saturate(170%);
```
Son libellé suit l'état : **Lancer** → **Ça se lance…** → **● En jeu** (point qui pulse). Il ne doit jamais y en avoir deux à l'écran. Les autres actions principales (« Analyser ce PC », « Ajouter à la bibliothèque ») reprennent la même recette.

### Bouton secondaire
Pilule en verre : `padding: 16px 22px`, 16 px, 600, fond `.08`, bordure `.2`.

### Chips
Pilule, 14 à 15 px, 600, avec le compteur en 12 px et `opacity: .7`. État actif : fond `.16`, bordure `.32`, texte plein.

### Jaquette
- Avec une image : l'image seule, sans texte par-dessus.
- Sans image : dégradé tiré de l'accent du jeu, titre en condensé en bas à gauche, source en petites capitales en haut à gauche.
- Jeu masqué : `grayscale(1) brightness(.7)`.
- Jeu en cours : point de l'accent de 10 px en haut à droite.
- Clic = sélectionner (accueil) ou ouvrir la fiche (bibliothèque). Double-clic = lancer.

### Barre de progression
4 px de haut (6 px dans le modal des succès), piste `rgba(255,255,255,.14)`, remplissage de l'accent avec lueur.

### Interrupteur (réglages)
48 × 28, piste de l'accent quand il est activé, `rgba(255,255,255,.16)` sinon. Curseur blanc de 22 px.

---

## 8. Images

- Les jaquettes (`library_600x900.jpg`) et les fonds héros (`library_hero.jpg`) viennent du CDN Steam, d'après l'App ID (table `APP` dans `konsol-proto.js`).
- Le **fond héros** est plein cadre, inversé horizontalement (`scaleX(-1)`) pour dégager le centre. Il est recouvert d'une vignette radiale (`rgba(5,6,12,.25)` au centre, `.9` sur les bords) et d'un dégradé vers l'encre en bas.
- Quand on change de jeu, les fonds se **croisent en fondu** (0,8 s). On ne coupe jamais net.
- Sans image, on revient au dégradé : l'écran doit rester beau sans aucune image.
- Pour ajouter un jeu : ajouter une ligne dans `R` (avec son accent) et son App ID dans `APP`.

---

## 9. Mouvement

| Mouvement | Durée | Courbe |
|---|---|---|
| Déplacement du carrousel | .55 s | `cubic-bezier(.2,.8,.2,1)` |
| Fondu entre fonds | .8 s | `ease` |
| Changement d'accent | .7 s | par défaut |
| Survol (jaquette, bouton) | .2 s | par défaut |
| Point « en jeu » | 1,6 s en boucle | opacité 1 → .3 |
| Halo « en jeu » | 3,2 s en boucle | opacité et échelle `.92` |

Le mouvement explique un changement d'état, il ne décore pas. Respecter `prefers-reduced-motion` et le réglage « Réduire les animations ».

---

## 10. Interactions et clavier

- **← →** : jeu précédent ou suivant dans le carrousel.
- **Entrée** : lancer le jeu sélectionné.
- **Ctrl K** : palette de recherche (jeux et actions), depuis n'importe quel écran.
- **Échap** : fermer l'overlay ouvert.
- **Double-clic** sur une jaquette : lancer le jeu.
- Toute nouvelle action doit aussi être accessible depuis la palette.

---

## 11. Rédaction

- Français, tutoiement exclu : on vouvoie (« Ajoutez », « Konsol revient quand vous quittez… »).
- Verbes d'action courts sur les boutons : *Lancer, Arrêter, Ajouter, Analyser, Masquer, Réafficher, Retirer, Renommer*.
- Les libellés reprennent ceux de konsol-ui : « Lancer », « Ça se lance… », « En jeu », « Voir les masqués », « Ajouter un jeu », « Réglages », « succès ».
- Dates relatives (« Hier », « Il y a 3 jours », « En septembre »), durées en heures (`86 h`).
- Pas de point d'exclamation, pas d'emoji.

---

## 12. Accessibilité

- Contraste du texte d'au moins 4,5:1 sur l'encre. Le texte secondaire ne descend pas sous `.5`.
- Ne jamais poser un texte de paragraphe sur une image sans voile.
- Focus visible : `outline: 2px solid rgba(255,255,255,.9)`, décalé de 3 px (règle globale du design system).
- Cibles cliquables d'au moins 40 px de haut.

---

## 13. À faire / à éviter

**À faire**
- Un seul élément lumineux par écran : la jaquette sélectionnée ou le bouton Lancer.
- Réutiliser la barre du haut, les recettes de verre et le bouton Lancer tels quels.
- Penser chaque nouvel écran avec ses trois états : avec des jeux, vide, en jeu.

**À éviter**
- Les fonds de couleur pleins, les dégradés criards, les bordures de couleur sur les cartes.
- Une 4e couleur d'interface : il n'y a que l'encre, le blanc et l'accent du jeu.
- Les icônes décoratives. Quelques glyphes simples suffisent (`+`, `‹ ›`, `×`).
- Changer la taille de la jaquette centrale ou casser la symétrie de l'accueil.
- Les listes denses façon tableur : pour ça, passer par la palette ou par la fiche.

---

## 14. Ajouter un écran : la checklist

1. Il garde la barre du haut, et un onglet au plus est actif.
2. Le fond reste l'image héros du jeu sélectionné, assombrie par un voile d'encre (`inkOp`) si l'écran est chargé en contenu.
3. Les titres sont en Bricolage 800 condensé, aux tailles de l'échelle ci-dessus.
4. Il y a au plus un bouton Lancer, et les actions secondaires sont en verre.
5. L'accent vient du jeu, aucune couleur n'est ajoutée.
6. Il est accessible au clavier et ses actions se retrouvent dans la palette.
7. Il est prévu dans les cas vide, chargement et en jeu.
8. Il a été vérifié à côté de l'accueil du carrousel : même lumière, même densité.
