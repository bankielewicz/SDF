---
schema: devforgeai/explore-brief/1
id: IDEA-001
phase: explore
status: decided
produced_by: exploring-ideas
consumes: []
open_questions: []
---

## Problem statement

A book club organiser loses the record of what the club has read within a month of each meeting, and rebuilds it from chat history every time a member asks.

## Target user

Volunteer organisers of book clubs that meet monthly
- Runs the club from a group chat and a shared calendar invite
- Keeps the reading list in a note on one phone
- Announces the next book two to four weeks ahead
- Hands the club to another member about once a year

## What they do today

| Current approach | Cost | Where it breaks |
|---|---|---|
| Group chat scrollback | 20 minutes per question | the chat is pruned and the early meetings are gone |
| A note on one phone | 5 minutes per meeting | the note leaves with the organiser who wrote it |
| A shared spreadsheet | 15 minutes per month | nobody but the organiser opens it |

## Why now

2026-01 — group chat platforms began pruning free-tier history at 90 days
2026-04 — public library lending APIs opened to personal accounts

## Competitor scan

| Name | URL | Approach | Price | Gap |
|---|---|---|---|---|
| Goodreads | https://www.goodreads.com | personal shelves with optional groups | free | the read record lives per member, not per club |
| The StoryGraph | https://app.thestorygraph.com | reading statistics for one reader | free tier, paid plus | no meeting date and no attendance |
| Bookclubs | https://bookclubs.com | club scheduling and polls | free tier, paid club plan | the read history sits behind the paid plan |

## Technology scan

| Capability | Candidate | Maturity | License | Source |
|---|---|---|---|---|
| Book metadata lookup | Open Library API | established | public domain data | https://openlibrary.org/developers/api |
| Calendar invitations | iCalendar | established | open standard | https://www.rfc-editor.org/rfc/rfc5545 |
| Offline-first storage | CRDT sync libraries | emerging | permissive | https://crdt.tech |

## Core flows

| ID | Actor | Trigger | Steps | Outcome |
|---|---|---|---|---|
| FLOW-001 | club organiser | monthly meeting is set | pick a book -> post the date -> collect who is coming | every member knows the book and the date |
| FLOW-002 | member | the date is posted | open the invite -> answer yes or no | the organiser has a head count |
| FLOW-003 | member | finished the book | mark it read -> leave a one-line note | the club sees who has finished |
| FLOW-004 | member | finished the book | open the club page -> mark the book as read | the club sees who has finished |

## Non-goals

The club does not sell or lend books.
The tool does not host discussion threads.
The tool does not rate or review a book.
The tool does not import a member's personal reading history.

## Success signal

clubs with a complete read history after three meetings | 6 of 10 clubs | 90 days from the first meeting logged

## Mockups

| Flow | Screen | Path | State |
|---|---|---|---|
| FLOW-001 | FLOW-001-01 | .devforgeai/explore/mockups/FLOW-001-01.html | default |
| FLOW-002 | FLOW-002-01 | .devforgeai/explore/mockups/FLOW-002-01.html | default |
| FLOW-003 | FLOW-003-01 | .devforgeai/explore/mockups/FLOW-003-01.html | default |
| FLOW-004 | FLOW-004-01 | .devforgeai/explore/mockups/FLOW-004-01.html | empty |

## Seed data

| Entity | Rows | Field count |
|---|---|---|
| book | 8 | 5 |
| meeting | 6 | 4 |
| member | 7 | 3 |

## Prototype

| Field | Value |
|---|---|
| Built | no |
| Path | - |
| Entry | - |
| Flows covered | - |
