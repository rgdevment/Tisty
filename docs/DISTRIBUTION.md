# How Tisty is distributed

Tisty reaches you in one of three ways, and they do not carry the same terms.
This page exists so nobody has to guess which one applies.

| What you have | Under what terms |
|---|---|
| The **source code**, from this repository | [AGPL-3.0](../LICENSE) |
| A **build you compiled yourself** from that source | [AGPL-3.0](../LICENSE) |
| A **signed build** downloaded from an app store or from the releases page | The terms below, plus the store's own |
| A build under a **commercial licence** | [COMMERCIAL.md](COMMERCIAL.md) |

## Why the store build is not simply AGPL

A licence is a grant of rights **to other people**. The copyright holder does
not grant rights to himself, so publishing this code under the AGPL-3.0 and
also publishing a signed build under different terms is not a contradiction —
it is the same author choosing twice.

That matters because the terms of service of the major app stores are not
compatible with the AGPL-3.0. Being able to publish there at all depends on the
copyright staying in one pair of hands, which is what the
[CLA](../CLA.md) is for.

**Nothing is withheld.** The store build is the same program, from the same
source, at the same version. What you pay for — where there is a price — is the
signing, the notarisation, the automatic updates and the store's own
convenience. If you would rather not, compile it yourself: the source is here,
it is complete, and it is free forever.

## The terms for a signed build

Mario Hidalgo G. (rgdevment) grants you a **non-exclusive, worldwide,
non-transferable licence** to install and use the signed build, on any number of
devices you own or control, for as long as you like, personally or at work.

You may not redistribute the signed build itself, decompile it, or remove its
signature. **None of that limits the source**: everything the AGPL grants over
the code stays granted, including your right to build, modify and redistribute
it under those terms.

The build is provided **as is**, without warranty of any kind, to the extent the
law where you live allows. That is the same warranty the AGPL-3.0 gives, worded
here so it is not a surprise.

## What the installer does to your machine

Little, and all of it reversible.

- It installs **under your own user account**, never machine-wide, so it never
  asks for an administrator.
- It puts the program in `%LOCALAPPDATA%\Programs\Tisty` on Windows — beside
  your other per-user programs, and deliberately **not** in the folder your
  tasks live in.
- The command line travels inside the installer, but **nothing is added to your
  PATH**. The app offers that from Maintenance, where it can ask first and can
  be undone. (An installer did it once and destroyed a PATH doing it: the tool
  that builds Windows installers reads at most 1024 characters and writes back
  what it managed to read.)
- **Uninstalling removes the program and leaves your data alone.** Wanting the
  program gone is not the same as wanting your history gone. If you do want it
  gone, the folder is yours to delete.

## What it does with your data

Nothing. Tisty stores your tasks on your own machine and speaks to no server of
ours — there is none. Synchronisation, when you turn it on, copies files into a
folder **you choose**, and what happens there is between you and whoever keeps
that folder in step. See [PRIVACY.md](../PRIVACY.md).

Stores report their own aggregate figures — downloads, crashes, versions — to
the developer account. That comes from the store, not from the program, and it
carries no personal data of yours.

## Where a build is authentic

Only two places:

- The **releases page** of this repository, where every artefact carries a
  SHA256 checksum and a [Sigstore](https://www.sigstore.dev/) attestation tying
  it to the commit, workflow and runner that produced it. Verify it with
  `gh attestation verify <file> --repo rgdevment/Tisty`.
- The **stores and package managers** listed in the README.

A build from anywhere else is not ours, however it is named.

## Questions

<github@apirest.cl>. If your situation is not covered here, ask before
assuming — a short email is cheaper than a wrong guess in either direction.
