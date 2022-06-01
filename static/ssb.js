function whoami() {
	$.get("/api/whoami", function(response){
		$("#profile_username").text(response.username)
		$('#form_username').attr('value', response.username)
		$("#profile_description").html(micromarkdown.parse(response.description))
		$('#form_description').text(response.description)
		$("#profile_pubkey").text(response.pubkey)
	})
}

function feed() {
	$.get("/api/posts", function(response){
		response.posts.forEach(createPost)
	})
}

function createPost(value, index, array) {
	console.log(value)
	$("#messages").append('<h3>'+value.author+'</h3><p><i>'+value.timestamp+'</p></i>')
}
