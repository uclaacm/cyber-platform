function slide() {
    let radios = [...document.getElementsByName('ws')]
    if (radios.some(x => x.checked)) {
        document.getElementById('deet').style.display = 'block'
        radios.forEach(x => {
            x.required = true;
            if (x.checked) {
                document.getElementById(x.value).style.display = 'block';
            }
            else {
                document.getElementById(x.value).style.display = 'none';
            }
        })
    }
}

function clear() {
    let radios = [...document.getElementsByName('ws')]
    radios.forEach((x) => {
        x.checked = false;
        x.required = false;

        document.getElementById(x.value).style.display = 'none';
    })
}

function refresh() {
    if (document.location.pathname === '/events') {
        if (document.location.hash === '') {
            if (document.referrer.split('/').splice(-1)[0] == 'events' && window.history.state) {
                let tiles = document.querySelectorAll('.workshop-left');
                tiles.forEach((x) => {
                    x.classList.add('workshop-up')
                })
            }
            clear();

        } else {
            document.getElementById(document.location.hash.replace(/#/gi, '')).checked = true;
            setTimeout(() => {
                slide();
            }, 200);
        }
    }
}

$(document).ready(function () {
    refresh()

    $("input").change(function (e) {
        slide();
        window.history.pushState({ event: true }, "", "#" + e.currentTarget.id.toLowerCase());
    })

    window.onhashchange = function (e) {
        if (e.oldURL.split('#').length > 1) {
            let tiles = document.querySelectorAll('.workshop-left');
            tiles.forEach((x) => {
                x.classList.add('workshop-up')
            })
        } else {
            let tiles = document.querySelectorAll('.workshop-up');
            tiles.forEach((x) => {
                x.classList.remove('workshop-up')
            })
        }
        refresh();
    }

    window.onpopstate = function (e) {
        refresh();
    };

})