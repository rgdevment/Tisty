# The Brainsty network

I had kept things in Tisty for years and had no way to see them together. I knew «Rina» cited the task for her vaccine and that the building minutes had created three tasks, but each tie lived inside its own file. Brainsty draws everything you keep, with what joins it, as one network: from afar, a map of your projects; up close, every document and every task with its neighbours.

This page covers opening it, what each shape and each line means, moving through it and filtering it. The ties it draws are the ones you make as you write: [[Linking documents|Linking-Documents]] and [[Tasks in documents|Tasks-in-Documents]]. Your agent reads the same network as an index: [[How an agent works|How-an-Agent-Works]].

![Tisty's window with the sidebar on the left and Brainsty marked in it; in the content panel, the network zoomed out: at the top the count, «589 nodes · 1252 connections», each folder and each list in its colour, documents as squares, tasks as circles and tags as purple dots, and at the bottom left the Legend, folded](https://raw.githubusercontent.com/rgdevment/Tisty/main/app/src-tauri/resources/guide/en/network.png)

## When it helps

- **Coming back to a project after weeks.** Open the network, search for the project and see at a glance its documents, its open tasks and the ones you already closed.
- **Finding what was left loose.** A document with no lines is one nothing names: maybe it lacks a link, or it is no longer needed.
- **Seeing what a tag touches.** Filter by `#rina` and her vaccines, her walks, the vet and this month's tasks remain.
- **Seeing what you touched last.** The **Touched** filter with **Last 7 days** leaves only what moved this week.

## Opening and leaving it

The network opens in the content panel of the main window, like any other view, with the sidebar in sight. That way you look at it without losing sight of your lists, and you leave it with a click on another view.

- **Brainsty**, in the sidebar under the task views, opens it, and stays marked while you look at it. With the sidebar folded, it is the network icon on the rail.
- It opens with the whole network in sight, turning like a globe (see [The living network](#the-living-network)). The document or task you were on stays as the starting point for the arrows; if you were on none, what you wrote last. At the top, the count: «589 nodes · 1252 connections».
- The first time, the nodes settle for a few seconds, as if let go, and stop by themselves within four seconds; a network of about 5,600 nodes is still in two or three.
- **The keyboard is its own** as soon as it is drawn: the arrows, `/` and `Enter` work without clicking on the network first.
- **It fits the panel.** When the panel changes width, for instance when you fold the sidebar, the network is drawn again to its size.
- **To leave**, pick another view in the sidebar, open something with **Open** or use `Esc` until it takes you back to where you were (see [With the keyboard](#with-the-keyboard)). Leaving stops everything the network ran: the drawing, the settling and its work in the background. Nothing is left spending.
- **Each time you come in, it starts afresh.** It is read again, with what you wrote meanwhile, and starts with no node picked, no search and no filters. All it remembers is whether you left the [legend](#the-legend) open.
- **It follows Tisty**, like any view: the light or dark theme and the language. In dark, the network takes the dark colours.

## What each thing is

| In the network | It is | How it looks |
| :-- | :-- | :-- |
| Square | a document or a page of a book | in its folder's colour; a page, smaller beside its document |
| Big square | a folder | amid its documents, tied to each one |
| Diamond | a list | amid its tasks, tied to each one |
| Circle | a task | in its list's colour; under way, bright; done, fainter; dropped or archived, faint and small |
| Purple dot | a tag | tied to everything that carries it |
| Grey dot | a picture or a file | tied to what uses it; up close, with its thumbnail |

Each folder and each list that holds something in the network is a node, amid what it holds. It pushes nobody: the network settles by its other lines and the folder or the list sits where its things ended up, so the network keeps its shape. Each one also tints its zone with its colour. They exist in the view alone: your agent does not see them as nodes in its index. The [[Agents|AGENTS-Folder]] folder has a colour of its own, coral, which no other folder can pick: its zone is tinted coral.

| Line | Ties | When it exists |
| :-- | :-- | :-- |
| Solid | a citation | a card of another document or of a task, alone in its paragraph |
| Dashed | a mention | a link inside a sentence to another document or to a task |
| Thin | the structure | a page and its book, a part and its task, a task and the document it was born in, an idea and its document, a folder and its documents, a list and its tasks |
| Thin and faint | a tag | what carries `#tag` and its dot |
| Thin, toward a file | a use | what shows a picture or attaches a file |
| Amber curve | a guess | two things that share uncommon words, a word with a typo or a translation, and that nobody tied; more opaque the more alike they are |

Every straight line is written somewhere, and can be explained. The amber curve is not: it is what Tisty guesses, never straight, solid or dashed, so it is never mistaken for anything written, and the node's card says why. [Alike](#alike) tells it.

## The legend

You do not need to learn these tables: **Legend**, at the bottom left of the network, says what each shape, each colour and each line of your network means. It comes folded, so it does not cover the network; a click on its title opens it and another folds it. Each computer remembers how you left it.

Open, it carries two switches at the top, **Travellers** and **Turning**, which [The living network](#the-living-network) tells. Below, it has four parts. Each sample is drawn with the same brush as the network, so what you see in the legend is exactly what you see in the network, in the dark theme too:

| Part | What it shows |
| :-- | :-- |
| **What it is** | the shape and the tone of each kind there is in your network: **Document**, **Page**, **Task**, **Tag**, **File**, **Folder** and **List**. A kind you do not have does not show |
| **Colour: its folder or list** | the coloured folders and lists that have something in the network, the fullest first. It shows six; **N more** opens the rest and **Fewer** hides them again. At the foot: «Without a colour, it takes its kind's tone.» |
| **State** | how a task **Under way**, a **Done** one and an **Archived** one look |
| **Lines, up close** | **Cites**, **Mentions**, **Part of (a page, a part, a folder or list)**, **Tagged** and **Uses a file**, only the ones in your network; and **Alike**, the amber curve, when Tisty guessed any. With the pointer on **Alike**, it tells you what they are |

For instance, Marisol has eleven coloured folders and lists. Her legend shows the six fullest, **Home**, **Work**, **Pets**, **Trips**, **Shopping** and **Health**, and below them **5 more**. A folder with no colour, like **Recipes**, is not on the list: its documents show in the documents' tone.

![The Legend open in the bottom left corner of the network: at the top the Travellers and Turning switches, with their dot lit; What it is with Document, Page, Task, Tag, File, Folder and List; Colour: its folder or list with Home, Work, Pets, Trips, Shopping and Health, «5 more» and «Without a colour, it takes its kind's tone.»; State with Under way, Done and Archived; Lines, up close with Cites, Mentions, Part of (a page, a part, a folder or list), Tagged, Uses a file and Alike in amber](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/help/en-legend.png)

## The living network

While you look at it, the network moves: it turns, and travellers walk its lines. If you would rather have it still, you turn that off from the legend.

- **It turns like a globe**, by itself, once a minute. What is behind looks smaller and fainter, and with the network turned the coloured zones hide and fewer titles show.
- **It stops when you need it still:** while the pointer is on the network's shape, a moment after you zoom in or out with the wheel, and while you have a node picked. Over the empty space around it, it keeps turning.
- **You turn it yourself** by dragging with the right button, or with `Shift` and the left one. A plain drag still moves the view, as always.
- **Travellers** are dots of light that walk the connections on long trips: they go from one node to the next every five seconds, leave a short trail and flash as they pass a tag or a node with many ties. They favour documents and pages, and never go through what the filters hide.
- **How many travellers** depends on the size of the network:

| Nodes in the network | Travellers |
| :-- | :-- |
| under 60 | 2 |
| 60 to 99 | 4 |
| 100 to 149 | 6 |
| 150 to 249 | 8 |
| 250 or more | 10 |

- **Two switches**, at the top of the open [legend](#the-legend): **Travellers** and **Turning**, each with a dot that lights up. Without **Turning**, the network lies flat; with both off, it is a still network. Each computer remembers how you left them. If your system asks for reduced motion, they start off.
- **It runs only while you see it.** Moving to another view stops it, like the rest of the network.
- **What it costs**, if you are curious: `Ctrl` + `Shift` + `M` shows beside the legend how many milliseconds each frame takes and how many frames a second it draws, for instance «2.10 ms a frame · 60 frames a second». Another `Ctrl` + `Shift` + `M` hides it.

For instance, Lucas opens Brainsty with 180 nodes: he sees eight travellers crossing the network as it turns. He rests the pointer on the **Work** zone to read a title and the globe stops; he moves it to the empty space and it turns again. To share his screen he prefers the network still: he opens the **Legend** and turns **Travellers** and **Turning** off, and next time on that computer it opens the same way.

## From afar and up close

The network changes what it shows with the distance, and each detail fades in as you come closer:

- **From afar**, with the wheel backwards: the nodes and their ties make the outline of a brain seen from the side, with no titles and faint lines. It shows where there is a lot and where there is little.
- **At middle distance**, as it opens: each folder and each list tints its zone, and only the most connected nodes carry a title.
- **Up close**: every title, each kind of line, tags as `#tag`, beside each picture its thumbnail and, inside each node, the icon of what it is: document, page, task (with a tick once done), tag, file, folder or list. The icon dims with what is not picked and costs nothing from afar. The amber curves show here, little by little, between two nodes in view; around the node you pick they show from any distance.

![Brainsty in the content panel, beside the sidebar, with the same network from afar: the nodes make the outline of a brain seen from the side, with no titles and faint lines, and the Legend folded at the bottom left](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/help/en-network-far.png)

The wheel zooms in and out; dragging moves you around, and dragging with the right button, or with `Shift`, turns the globe. At the first touch the network stops following the point it started on.

## Picking, looking and opening

**A click on a node** lights it up with its neighbours, dims the rest and opens its **card**, on the right. The card does not show the document's text: it shows its facts, so you know what it is, when it is from and what it joins before you open it. At the top go its title and a line with what it is, its folder or its list and how many connections it has: «Document · Pets · 6 connections».

Below come its facts, by what you picked:

| You pick | The card shows |
| :-- | :-- |
| A document | **Last written**, **Created**, **Size**, with its weight and how many words it has, and **Language**, like «EN» |
| A page of a book | the same, and first **Page of**, with its book's title |
| A task | **State**, like «Under way · Due soon»; **Day**, **Deadline**, **Part of**, with its parent task, **Parts**, with how many it has, **Steps**, like «1 of 3 done», and **Source**, if it came with one |
| A tag, a file, a folder or a list | only the line at the top and its connections |

A task's state adds, after a dot, how its date is going: **Ahead**, **For today**, **Late**, **Underway**, **Due soon** or **Overdue**. A row with nothing to say does not show: a task with no deadline shows no **Deadline**.

Then, in this order, what there is:

- **Tags**, each like `#rina`.
- **Connections**, counted by kind: «2 cites 3 mentions».
- **Pointing here**: the documents and tasks that cite it or mention it, up to six, and «N more» when there are others. A click on one picks it in the network and opens its card; one that is not in the network, like what was left out past the node cap, is read but cannot be picked.
- **Alike**, in amber, when Tisty guesses any: each with its title and its why, «Tisty guesses: they share *quote* and *roof*». A click on the title picks it in the network. [Alike](#alike) tells it.
- **Link**, on a document, a page or a task: its address and **Copy link**, which then says «Link copied». It is the same link you copy from a document's panel, to paste in a chat, in your browser or hand to your agent. A document's and a task's look like this:

```text
tisty://doc/pets/rina?id=…
tisty://task?id=…
```

At the foot, **Open**. The cross at the top, **Close the preview**, or `Esc`, lets go of the node and the whole network shows again.

- **Open**, `Enter` or **a double click** on the node opens it in the same window, in the network's place: a document in the Library, a task beside its list, a tag in its view, a folder in **Documents** and a list in **Lists**. To come back to the network, another click on **Brainsty**; it opens whole again, and what you opened stays as the starting point for the arrows.
- **A picture or a file** has a card but no **Open** and no **Link**: it opens from the document or the task that uses it, its neighbour in the network.

![«Rina» picked up close in Brainsty, in the content panel beside the sidebar: her neighbours with titles, the rest dimmed and, on the right, her card: «Document · Pets · 6 connections», the rows Last written, Created, Size with its words and Language EN, the tag #rina, Connections by kind, Pointing here with what cites her, Alike with «Saturday walks» and «Tisty guesses: they share rina and park», the Link with Copy link and the Open button; in the network, an amber curve joins Rina to «Saturday walks»](https://raw.githubusercontent.com/rgdevment/Tisty/main/app/src-tauri/resources/guide/en/network-near.png)

## Searching

The search box sits at the top left, **Search the network**; `/` focuses it from anywhere in the network. Type part of a title: accents and capitals do not matter. The list of what matches shows below, each with what it is («Document», «Task», «Tag»…): first what starts the same way, then what has a word that starts that way, and last the rest. `Enter` picks the first, or a click the one you want: the network centres on it and opens its card. When nothing is called that, it says «Nothing by that name».

## Filtering

The funnel beside the search box, **Filters**, opens what can be left out. Each option is a button that turns on and off, and the network is drawn again with what remains:

- **Show**: **Documents**, **Pages**, **Tasks**, **Tags**, **Files**, **Folders**, **Lists** and **Alike**. **Alike** starts on; off, the network draws no amber curve and the card lists none. A folder or a list hides by itself when the filters hide everything it holds: with only **Tasks**, the folders go and the lists stay.
- The state of tasks: **Under way**, **Done** and **Archived**.
- **Touched**: **Any time**, **Last 7 days**, **Last 30 days** or **Last year**, by the last day each thing changed.
- **Tag**: pick one from the list and only what carries it remains; **Any tag** takes it off.

With a filter on, the funnel is tinted and **Show everything** appears, which brings the whole network back.

![Brainsty in the content panel with Filters open: Documents, Tasks and Alike on, Under way, Last 30 days and the rina tag, the funnel tinted, Show everything in sight and the network with what remains](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/help/en-filters.png)

## With the keyboard

| Key | What it does |
| :-- | :-- |
| `Tab` | go round the search box, **Filters**, the card and the network |
| Arrows | pick a node and walk: first to a neighbour in that direction, otherwise to the nearest one; an amber curve is not a path. With no node picked, the first one sets out from the document or task you were on |
| `Enter` | open the picked node, in the network's place |
| `/` | search |
| `Esc` | let go in layers: first the search, even when it is empty; then the node; and the last one takes you back to the view you were on |
| `Ctrl` + `Shift` + `M` | show or hide what each frame costs |

The line of keys at the foot of the network sums it up: «Arrows move between nodes, Enter opens, / searches, Esc lets go, right drag or Shift turns».

After you pick a search result, or leave the search with `Esc`, the arrows and `Enter` walk the network again, with no mouse. So `/`, `rina`, `Enter` and `Enter` again open Rina's document without leaving the keyboard.

An `Esc` you type in a field of the sidebar, like its search box, belongs to that field: it lets go of nothing in the network and does not take you out of it.

## An example: back to Rina's things

You have not touched anything about Rina for a month and want to know how it is going. You are on the building minutes.

1. **Brainsty** in the sidebar. The network opens in the panel, in place of the minutes, whole and turning, with its travellers.
2. `/`, `rina`, `Enter`. The globe stops, the network centres on «Rina», the document in the **Pets** folder, and opens its card: «Document · Pets · 6 connections», last written a month ago, in English.
3. You come closer with the wheel: her neighbours show with titles, «Vet» joined by a dashed line (you mention it in a sentence), the solid card of «Rina's yearly vaccine», the photo of the walk with its thumbnail, the big square of **Pets**, amid its documents, and an amber curve to «Saturday walks», which nobody linked. Rina's card says why, under **Alike**: «Tisty guesses: they share *rina* and *park*».
4. A click on «Rina's yearly vaccine»: its card says «Under way · Due soon», **Deadline** on Friday and **Steps** «1 of 3 done», and under **Pointing here** is «Rina», which cites it. **Copy link** gives you its link to pass on to Yisset, who is taking her.
5. **Open**: the task opens beside its list, in the network's place, and there you tick «Book a slot».
6. Another click on **Brainsty**: the network opens whole again; the task stays as the starting point for the arrows. **Filters**, **Tag** `rina` and **Touched** **Last 30 days**: her two documents and the tasks under way that moved this month remain.
7. `Esc` until you are back: first it lets go of what you picked and the last one takes you back to the task. Filters are not kept: next time the network starts whole.

## Alike

Two things can be about the same without anyone having tied them: the task «Ask for a quote on the roof» and the document «House repairs», which talks about the roof and its quote. Brainsty notes them as **alike**, apart from the lines you wrote.

Tisty looks for things alike with three clues, in this order. Each clue only fills the places the one before left free, up to three alike for each thing, and each guess keeps what says why.

### Shared words

Two things are alike when they share at least two **weighty words**: the kind that shows up in few of the things you keep, in 5 % of them or fewer. *Quote* and *roof* weigh; *week* or *make*, which are everywhere, do not. Words are compared without accents or capitals, so *Roof* and *roof* are the same.

It works across languages too when what they share are names: a task in English and a document in Spanish that both talk about *Folio* and *Glama* are alike.

### A typo

You type fast and jot «La imrpesora nueva no imprime en color» (the new printer prints no colour). A week later, Jaime asks you to «Instalar la impresora nueva». They share only *nueva*, but *imrpesora* is one change away from *impresora*, and Tisty ties them: «Tisty guesses: they share *impresora ≈ imrpesora* and *nueva*». It works the same in English, for a word of six letters or more.

- **One change** is a letter too many, one too few, a different one or two neighbouring letters that swapped places, like *rp* for *pr*.
- **Only words of six letters or more.** *Cast* and *cost* are two words, not a mistake.
- **Never words with digits.** *chapter1* and *chapter2* were written apart on purpose.
- Of the two spellings, the one you use more in what you keep is the one that counts as right.

### A translation

You jot «Pagar la cuenta del agua» and, another day, your agent jots in English «Pay the water bill». They are about the same thing and share not one word. Tisty ties them with a Spanish–English dictionary: «Tisty guesses: they share *pagar ≈ pay* and *agua ≈ water*». The same goes for «Arreglar la gotera del techo» and «Fix the roof leak»: *gotera ≈ leak* and *techo ≈ roof*.

- **It is not AI.** It is a fixed list of 41,140 pairs of words, made by linguists: it comes from the [Open Multilingual Wordnet 2.0](https://github.com/omwn/omw-data), with the Multilingual Central Repository 3.0 for Spanish and Princeton WordNet 3.0 for English. It is a file of about 760 KB (777,917 bytes) that comes inside the installer; compressed there, it adds about 0.3 MB to the download. It is never looked up online. See [No AI](Privacy#no-ai).
- **One pair per word**, the one of its most used sense. So sometimes it is not the one you expect: *cuenta* gives *tally*, not *bill*. In the example above, what ties them is *pagar* and *agua*.
- **A word written the same in both languages**, like *hotel*, is not on the list: it already ties as a shared word.
- **It only ties two different languages:** never two things in Spanish nor two in English, even when they say the same with other words.
- **The language of each thing** is the one the document declares, if it says. If not, Tisty guesses it from words like «el», «la», «de» or «the», «of», «to», and from accents and the ñ. A thing in English is never read as Spanish.
- **When it cannot guess, it does not tie it by dictionary.** A task that only says «Router», or «Rina's yearly vaccine», carries none of those words. Shared words and typos still tie it.
- **Only the app reads the dictionary**, in its low-priority pass. Your agent's gate does not load it: your agent sees on the card the guesses the app already kept.

### What holds for all three

A typo or a translation counts like a shared word: two things are alike only when they share at least two weighty words, and at least one of them written differently on each side. In a translation, a word each side wrote differently, like *tarjeta* and *card*, counts even when it shows up in as much as 25 % of what you keep, not only 5 %.

- **Three alike at most for each thing**, and each end must be among the other's three closest.
- **The bar comes from what you keep**, not from a fixed figure: a guess must beat what one pair in a thousand reaches in your base.
- **What is already tied is not guessed**: when a line in the network already ties them (a citation, a mention, a part and its task, the document a task was born in), or when they are a page and its book, a document and its version in another language, or a task and its page.
- **A document in another language** has a place of its own among the three, so what it shares with another language is not crowded out by kin in its own.
- **Each thing keeps up to 256 words**, the ones said most. Bare numbers and long runs of letters and digits, like an id or a hash, do not count.

**When.** A low-priority pass, in the background, looks every 30 seconds and waits for you to stop writing, five minutes at most, before comparing. The words of what changed are kept at once. When a new version of Tisty changes Brainsty's base, the base is built again once, on the first start: meanwhile the network says «Brainsty is still reading what you keep. Open it again in a moment.».

**What never goes through it.** Hidden and dropped tasks, and your whole [Diary](Documents#diary), keep no words, even when you open the Diary to agents. Hiding a task takes its guesses away in the same round.

**Where they show:**

- **In the network**, as an amber curve: up close between two nodes in view, and around the node you pick from any distance. How opaque it is is measured among the guesses in your base: the most opaque is the most alike.
- **On the card**, in its **Alike** section, apart from the connections. They show as the base keeps them, with no accents and in lowercase. A shared word goes alone, *quote*; a typo or a translation, with what each side wrote, *impresora ≈ imrpesora* or *pagar ≈ pay*.
- **On the card your agent reads**, under `guessed`, apart from everything someone wrote. Each guess brings its title, its score and what they share, and the rules ask it to read both before joining anything. A pair comes as `a≈b`, with no spaces: it is a translation or a typo, never a word both of them wrote:

```json
> brain {"task": "…Ask for a quote on the roof…"}
{ "guessed": [
    { "id": "…", "kind": "doc", "title": "House repairs", "score": 0.42, "shared": ["quote", "roof"] } ] }
"Guessed alike by Tisty, linked by no one: \"House repairs\" […] (shares quote, roof)."

> brain {"task": "…Pay the water bill…"}
{ "guessed": [
    { "id": "…", "kind": "task", "title": "Pagar la cuenta del agua", "score": 0.61, "shared": ["pagar≈pay", "agua≈water"] } ] }
"Guessed alike by Tisty, linked by no one: \"Pagar la cuenta del agua\" […] (shares pagar≈pay, agua≈water)."
```

![The card of «Pay the water bill» up close in Brainsty: «Task · Home» with its connections, State «Under way · For today», and under Alike «Pagar la cuenta del agua» with «Tisty guesses: they share pagar ≈ pay and agua ≈ water»; in the network, an amber curve joins the two tasks](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/help/en-twin.png)

A guess shows only when both its ends are in sight: in the network, both must be drawn; for an agent, both must pass its gate, so a hidden task is never named through a guess. A guess adds no connection to the count at the top and does not count towards the 12,000-node cap.

## Good to know

- **It builds itself, on this computer.** Tisty reads how your things are tied in the background, without slowing what you do. A link you just wrote reaches the base a few seconds later.
- **The network is read when you come in.** What you wrote in another view is there the next time you come into **Brainsty**. A new guess takes a little longer: it waits for the low-priority pass.
- **Just installed**, while it reads everything for the first time, the network says «Brainsty is still reading what you keep. Open it again in a moment.». With nothing kept: «Nothing to draw yet: write a document or a task and it shows up here.».
- **Up to 12,000 nodes and 40,000 connections.** Past that, the network keeps what you touched last, its neighbours, your documents and the tasks under way, and says at the top how much was left out: «312 nodes and 940 connections left out». Past 6,000 nodes, nodes shrink so the whole network takes the same room in the panel.
- **What is hidden does not show.** A task you hid is not drawn, neither as a node nor as the end of a line, and a tag or a file only it carried goes with it. A dropped task does show, faint and small.
- **Nothing leaves the computer.** The network and the guesses are worked out on each computer and rebuild themselves: they are not synced, and they do not go to Folio, a `.tistyx` or an export. What travels is your documents and tasks, and each computer draws it again.
- **Without a dedicated graphics card** the network is drawn just the same; zooming in and out costs a little more.

## Where next

- [[Linking documents|Linking-Documents]]: the dashed and solid lines between documents.
- [[Tasks in documents|Tasks-in-Documents]]: the lines between a document and its tasks.
- [[How an agent works|How-an-Agent-Works]]: the same network, read by your agent.
- [[Documents]]: folders and tags, which give the colours and the zones.
