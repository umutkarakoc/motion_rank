window.onload = function() {
    document.body.addEventListener("open_file_picker", function(ev){
        let input = document.createElement('input');
        input.type = 'file';

        let {AuthorizationExpire, AuthorizationSignature, LibraryId, VideoId} = ev.detail;

        input.onchange =  async () => {
            let files = Array.from(input.files);
            let file = files[0];
            let title = file.name.split(".").slice(0, -1).join(".");
            let parts =  location.pathname.split("/");
            let project_id = parts[parts.length - 1]

            await htmx.ajax("POST", `/project/${project_id}/video/${VideoId}`, {swap: "none", values: {title}});
            setTimeout(() => {
                htmx.ajax("GET", window.location.pathname, {target: "body", swap: "outerHTML"});
            }, 1000);

            let tusupload = new tus.Upload(file, {
                endpoint: "https://video.bunnycdn.com/tusupload",
                retryDelays: [0, 3000, 5000, 10000, 20000, 60000, 60000],
                headers:  {AuthorizationExpire, AuthorizationSignature, LibraryId, VideoId},
                metadata: {
                    filetype: file.type,
                    title: title,
                },
                onError: function (error) { rej(error) },
                onProgress: function (bytesUploaded, bytesTotal) {
                    try {
                        let progress = Math.min(100, (bytesUploaded / bytesTotal * 100).toFixed(2));
                        document.querySelector(`#video_${VideoId} progress`).value = progress;
                    } catch (err){
                        console.error(err)
                    }
                },
                onSuccess: function () {
                    setTimeout(() => {
                        htmx.ajax("GET", window.location.pathname, {target: "body", swap: "outerHTML"});
                    }, 3000);
                }
            } ) ;
            tusupload.start()
        };
        console.log("hello")
        input.click();
    })
}