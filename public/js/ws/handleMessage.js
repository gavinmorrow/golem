import ChatMessage, { messageForm } from "../custom-elements/ChatMessage.js";

let joined = [];
const nickList = document.getElementById("nick-list");

function handleMessage(message) {
	console.log("Got message:", message);
	switch (Object.keys(message)[0]) {
		case "Authenticate":
			handleAuthenticate(message);
			break;
		case "NewMessage":
			handleNewMessage(message);
			break;
		case "Messages":
			handleMessages(message);
			break;
		case "Join":
			handleJoin(message);
			break;
		case "Leave":
			handleLeave(message);
			break;
		case "Update":
			handleUpdate(message);
			break;
		case "Error":
			handleError(message);
			break;
		default:
			console.error("Unknown message type:", message);
	}
}

function handleAuthenticate(message) {
	if (message.Authenticate.success) {
		console.log("Successfully authenticated!");

		// Disable the login button (already logged in)
		document.getElementById("show-login").setAttribute("disabled", "");

		// Show send message form
		messageForm.style.display = "block";

		// Update change name form
		document.getElementById("new-name").value = joined.filter(
			j => j.id === message.Authenticate.presence_id
		)[0].name;
	} else {
		console.error("Failed to authenticate:", message.Authenticate.error);
	}
}

function handleNewMessage(message) {
	console.log("New message:", message.NewMessage);

	message = message.NewMessage;

	const chat = document.getElementById("chat");
	makeMessageElem(message).insert();
}

function handleMessages(message) {
	console.log("Messages:", message.Messages);

	const messages = message.Messages.sort((a, b) => {
		a = BigInt(a.id);
		b = BigInt(b.id);
		if (a < b) return -1;
		if (a > b) return 1;
		return 0;
	});

	// Add to DOM
	for (const message of messages) {
		makeMessageElem(message).insert();
	}
}

function handleJoin(message) {
	console.log("Someone joined!", message.Join);
	joined.push(message.Join);

	// Add to nick list
	const name = message.Join.name;
	const elem = document.createElement("div");
	elem.textContent = name;
	elem.setAttribute("data-presence-id", message.Join.id);
	nickList.appendChild(elem);
}

function handleLeave(message) {
	console.log("Someone left!", message.Leave);

	joined.filter(j => j.id !== message.Leave.id);

	// Remove from nick list
	const id = message.Leave.id;
	const elem = nickList.querySelector(`[data-presence-id="${id}"]`);
	elem.remove();
}

function handleUpdate(message) {
	console.log("Update:", message.Update);

	joined = joined.map(j => {
		if (j.id === message.Update.id) {
			console.log("Updating:", j, message.Update);
			return message.Update;
		}
		return j;
	});

	// Update nick list
	const id = message.Update.id;
	const elem = nickList.querySelector(`[data-presence-id="${id}"]`);
	elem.textContent = message.Update.name;
}

function makeMessageElem(message) {
	return new ChatMessage(message);
}

function handleError(message) {
	console.error("Error:", message.Error);
}

export default handleMessage;
