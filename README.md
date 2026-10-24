    # SigmaOS-menus

    Application menu & menu database

    Part of the **SigmaOS** sovereign operating-system ecosystem — a multi-repository
    distribution modelled on Linux Mint and Omarchy Linux. See the parent
    [`SigmaOS`](https://github.com/AaryanSinghChauhan09/SigmaOS) repository for the
    full component index.

    ## Upstream inspirations
    - https://github.com/linuxmint/cinnamon-menus
- https://github.com/linuxmint/mintmenu

    ## Design principles applied
    - **Abstraction**
- **Least Astonishment**
- **Fail-Fast**
- **KISS**
- **DRY**
- **Deterministic Testing**

    ## Usage
    Verbs: `categories, search, edit-entry, rebuild`

    ```
    $ sigma-menus categories
$ sigma-menus search fire
$ sigma-menus edit-entry firefox --name "Firefox Web Browser"
$ sigma-menus rebuild

    ```

    ## Build & test
    ```
    make build
    make test
    ```

    ## Licence
    GNU General Public License v3.0 (FOSS).
