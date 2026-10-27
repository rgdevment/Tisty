# Natural language

You type a sentence; Tisty takes the date out of it, leaves the sentence readable, and shows you what it understood **before** anything is saved — as chips you can correct with one click.

It is **rules running on your machine**: no model, no request, no cloud. It reads English and Spanish, following the language you chose. What it cannot read it leaves alone rather than guess, and when it reads something it was not sure of, it dates it and says so out loud, so you can correct it.

The examples below come from the parser's own test cases, which define exactly how far it goes. They assume today is **Wednesday 5 August 2026, 09:00**.

![Type it the way you say it: tags, lists, priority, dates and repeats are read from plain words](https://raw.githubusercontent.com/rgdevment/Tisty/main/assets/store/en-02-tasks.png)

## Days

| You write | Date |
| :-- | :-- |
| `review the report today` | Wed 5 Aug |
| `review the report tomorrow` | Thu 6 Aug |
| `deploy the day after tomorrow` | Fri 7 Aug |
| `standup on monday` | Mon 10 Aug |
| `standup next tuesday` | Tue 11 Aug |
| `standup on wednesday` | Wed 12 Aug — today is Wednesday, so it means the next one |
| `renew the certificate in 3 days` | Sat 8 Aug |
| `check back in 2 weeks` | Wed 19 Aug |
| `deliver the report within 3 days` | Sat 8 Aug |
| `meeting monday 15` | 15 Aug — the weekday names it, the number dates it |
| `meeting monday 3` | 3 Sep — a day of the month already past rolls to next month |

## Dates

| You write | Date |
| :-- | :-- |
| `pay on august 15` / `pay on aug 15` | 15 Aug 2026 |
| `ship it on september 3rd` | 3 Sep 2026 |
| `pay on jul 30` | 30 Jul **2027** — no year written and July is gone, so the next one |
| `pay on 2026-07-30` | 30 Jul 2026 — a written year is never rolled forward |
| `pay on 15/8` · `pay on 15/8/2026` | 15 Aug 2026 |
| `deliver on 24/12/26` | 24 Dec 2026 — a two-digit year belongs to this century |

> [!WARNING]
> Slash dates are read **day/month**. `pay on 8/15` (the US order) is not read: month 15 does not exist, so nothing is invented — and nothing is read either.

## Periods

| You write | Date |
| :-- | :-- |
| `tidy the desk this week` | Sun 9 Aug — the end of the week |
| `rest this weekend` | Sat 8 Aug |
| `close the books end of month` | 31 Aug |
| `review it next week` | Sun 16 Aug — the end of the period |
| `review it next month` | 30 Sep — the last day of next month |

## Times

| You write | Time |
| :-- | :-- |
| `deploy tomorrow at 10` | 10:00 |
| `deploy tomorrow at 10:30am` | 10:30 |
| `deploy tomorrow at 4pm` | 16:00 |
| `lunch tomorrow at noon` | 12:00 |
| `call the bank at 3` | **15:00** — from 1 to 6, the afternoon is read, and said out loud |
| `call the bank at 7` | 07:00 — from 7 on, the digits are taken as written |
| `call the bank at 03:00` | 03:00 — a leading zero is 24-hour notation |
| `call at 8 in the evening` | 20:00 |
| `call at 5 in the morning` | 05:00 |
| `call at 12 at night` / `meeting at 12 am` | 00:00 |

A bare number is only a time behind «at»: `turn the volume to 12` stays a title, and so does `upgrade to 10.2 tomorrow` (10.2 is a version).

## Date versus deadline

A **date** is when you plan to do it. A **deadline** is when it stops being possible. Four words open a deadline: **before**, **by**, **due** and **until**.

| You write | Reads as |
| :-- | :-- |
| `send the report on friday` | date: Fri 7 Aug |
| `send the report by friday` | deadline: Fri 7 Aug |
| `renew the domain due august 30` | deadline: 30 Aug |
| `send the report until friday` | deadline: Fri 7 Aug |
| `ship the report tomorrow before friday` | date: tomorrow **and** deadline: Friday |

## Repeating

Naming a day fixes the series to that day; an interval counts from when you last did it — which is what a habit wants.

| You write | Repeats |
| :-- | :-- |
| `take out the bins every tuesday` | every week, on Tuesday, whether or not it went out last week |
| `standup every monday at 9` | every Monday at 09:00 |
| `water the plants every 3 days` | 3 days after each time you do it |
| `stretch every day` / `back up the laptop daily` | daily, from the doing |
| `check the backups every 2 weeks` | every 2 weeks |
| `pay the rent every month` / `monthly` | monthly |
| `renew the domain every year` / `annually` | yearly |
| `take the pill every day until september 30` | daily, ending 30 Sep — «until» on a series says when it ends |
| `take the pills every day at 10 for 30 days` | daily at 10:00; «for 30 days» stays in the title |

A duration is not a date: `run the tests for 30 days` and `a course of 3 months` stay as written. And «daily», «weekly», «monthly» in the middle of a phrase are adjectives: `write the daily report` or `cancel my monthly subscription` repeat nothing.

## Markers

| You write | Means |
| :-- | :-- |
| `#backend #urgent` | tags |
| `@work` | the list |
| `!do` · `!schedule` · `!delegate` · `!minor` | the quadrant ([[Tasks]]); also `!now`, `!decide`, `!wont` |
| `!important` | *schedule* — important without urgency |
| `!DO` | the same as `!do` |

`!1` and `!urgent` set nothing: the quadrants are not a scale, and «urgent» fits two of them.

## When it leaves words alone

The parser is careful not to eat your title:

| You write | Why |
| :-- | :-- |
| `review monday's report` | «monday's» names the report |
| `tomorrow's meeting` | the possessive names the meeting |
| `meeting in the morning` | a part of the day, not a day |
| `ship it 3 days ago` | the past has no date here |
| `read 2/3` · `ratio 16/9` | a chapter or a fraction — a slash date needs a word in front |
| `set up 24/7 support` | «24/7» is a phrase, not a date |
| `buy milk tomorrow and bread on friday` | two days: the last one wins, the first stays as text |

### Escape it with quotes

Wrap the whole sentence in quotes and nothing is interpreted:

```text
"standup on monday"
```

## In Spanish

The same rules, in Spanish: `mañana a las 10`, `el viernes`, `pasado mañana`, `en 3 días`, `antes del viernes`, `hasta el 30 de agosto`, `cada martes`, `todos los lunes a las 9`, `cada 3 días`, `diariamente`.

| Quadrant | English | Spanish |
| :-- | :-- | :-- |
| Do | `!do`, `!now` | `!hacer`, `!ahora` |
| Schedule | `!schedule`, `!decide`, `!important` | `!planificar`, `!decidir`, `!importante` |
| Delegate | `!delegate` | `!delegar`, `!delega` |
| Minor | `!minor`, `!wont` | `!prescindible`, `!descartable` |
