# Flatpak

`io.github.bewzaalex.kadence.yml` builds Kadence on the freedesktop runtime with the Rust SDK
extension. Flatpak builds have no network, so `cargo-sources.json` lists every crate from
`Cargo.lock` as a download; cargo then builds offline from them.

Kadence isn't on Flathub. Flathub doesn't accept manifests written with AI help, so a Flathub
submission needs its own manifest, written and submitted by a person.

## Build and test

You need the builder, the freedesktop SDK and its Rust extension as flatpaks (the extension is
a download of about 560 MB):

```sh
flatpak install --user flathub org.flatpak.Builder org.freedesktop.Sdk//25.08 \
    org.freedesktop.Sdk.Extension.rust-stable//25.08
```

Build in `build/flatpak/`, so everything the builder writes stays there, its cache in `.flatpak-builder`
included. The first build downloads all crates in `Cargo.lock` (later builds reuse them):

```sh
mkdir -p build/flatpak && cd build/flatpak
flatpak run org.flatpak.Builder --user --force-clean --repo=repo \
    --compose-url-policy=full --mirror-screenshots-url=https://dl.flathub.org/media/ \
    app ../../packaging/flatpak/io.github.bewzaalex.kadence.yml
flatpak install --user ./repo io.github.bewzaalex.kadence
flatpak run io.github.bewzaalex.kadence
```

Check the manifest and the build against Flathub's rules, still in `build/flatpak/`:

```sh
flatpak run --command=flatpak-builder-lint org.flatpak.Builder manifest ../../packaging/flatpak/io.github.bewzaalex.kadence.yml
flatpak run --command=flatpak-builder-lint org.flatpak.Builder repo repo
```

Both should pass with no errors. The two URL options put the screenshots and icons in the catalog
with their Flathub addresses, as Flathub's own builds do.

The manifest builds the release tag, not your working copy. To try local changes, replace the `git`
source of the `kadence` module with:

```yaml
- type: dir
  path: ../..
  skip: [.git, build, target, design/reference]
```

## Crates

When `Cargo.lock` changes, regenerate the crate list and commit it with the lock file:

```sh
python3 packaging/flatpak/cargo-sources.py
```

## A new release

1. Add the release to `assets/io.github.bewzaalex.kadence.metainfo.xml`, and point the screenshot URLs
   at the new tag. Update `docs/screenshots/` if the panel changed
   (`kadence --render docs/screenshots/backlit.png light`, `KADENCE_SCALE=1.5`).
2. Tag it (`vX.Y.Z`) and publish a GitHub release.
3. In the manifest, set `tag` and `commit` of the `kadence` module to the new tag.
4. Build and check it as above, then make the bundle in `build/flatpak/` and attach it to the GitHub release:

   ```sh
   flatpak build-bundle --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo \
       repo kadence-X.Y.Z-$(flatpak --default-arch).flatpak io.github.bewzaalex.kadence master
   ```

5. Update the version in the file name in the top README's Flatpak section.

## Clean up

From the top folder of the repository:

```sh
flatpak uninstall --user io.github.bewzaalex.kadence
rm -rf build/flatpak
```
