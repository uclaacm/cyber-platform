function slide() {
    let radios = [...document.getElementsByName('ws')]
        if (radios.some(x => x.checked)) {
            console.log(radios)
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

$(document).ready(function () {
    $("input").change(function (e) { slide() })
})
    