const messageForm = document.getElementById("msg-form");

class ChatMessage extends HTMLElement {
	static messages = [];

	/**
	 * @type {Object}
	 * @property {string} id
	 * @property {string} author
	 * @property {string} parent
	 * @property {string} content
	 */
	message;

	children = [];

	constructor(message) {
		super();

		this.message = message;

		// Create shadow DOM
		const shadow = this.attachShadow({ mode: "open" });

		const author = document.createElement("address");
		author.textContent = `[${message.author_name}]`;
		author.setAttribute("rel", "author");

		const space = document.createTextNode(" ");

		const content = document.createElement("span");
		content.textContent = message.content;

		const style = document.createElement("style");
		style.textContent = `
		@import url("/css/defaults.css");

		address, span {
			margin-left: 0;
			display: inline-block;
		}

		address {
			color: var(--color-text-author);
			font-style: normal;
		}

		chat-message, form {
			margin-left: 1em;
			display: block;
			position: relative;
		}

		chat-message::before, form::before {
			content: "";
			display: block;
			width: 1px;
			height: 100%;
			background-color: gray;
			left: -1em;
			position: absolute;
		}
		`;

		shadow.appendChild(author);
		shadow.appendChild(space);
		shadow.appendChild(content);
		shadow.appendChild(style);

		ChatMessage.messages.push(this);

		this.addEventListener("click", event => {
			if (event.target === this && this.shadowRoot.contains(messageForm)) {
				// Allow event to propagate to parent
				return;
			}

			this.insertMessageInput();
			event.stopPropagation();
		});
	}

	connectedCallback() {
		this.setAttribute("class", "message");
		this.setAttribute("id", `msg-${this.message.id}`);
		this.setAttribute("data-author", this.message.author);
		this.setAttribute("data-parent", this.message.parent);
		this.setAttribute("data-content", this.message.content);
		this.style.display = "block";
		this.style.position = "relative";
	}

	addChild(child) {
		// Add to DOM
		this.shadowRoot.appendChild(child);

		// If needed, move form down
		if (this.shadowRoot.contains(messageForm)) {
			this.shadowRoot.appendChild(messageForm);
		}

		this.children.push(child);
	}

	insert() {
		const parent = ChatMessage.messages.filter(
			msg => msg.message.id === this.message.parent
		)[0];

		if (parent == null) {
			throw new Error(`Parent message not found: ${this.message.parent}`);
		}

		parent.addChild(this);
	}

	insertMessageInput() {
		this.shadowRoot.appendChild(messageForm);
		messageForm.querySelector("#msg-input-parent").value = this.message.id;
	}
}

customElements.define("chat-message", ChatMessage);

class TopLevelChatMessage extends ChatMessage {
	constructor() {
		super({
			id: "0",
			author_name: "",
			content: "",
		});

		// Remove all children
		while (this.shadowRoot.firstChild) {
			this.shadowRoot.removeChild(this.shadowRoot.lastChild);
		}

		const style = document.createElement("style");
		style.textContent = `
		@import url("/css/defaults.css");

		chat-message, form {
			margin-left: 0;
			display: block;
			position: relative;
		}

		chat-message::before, form::before {
			content: "";
			display: none;
			width: 1px;
			height: 100%;
			background-color: gray;
			left: -1em;
			position: absolute;
		}
		`;

		this.shadowRoot.appendChild(style);
		this.insertMessageInput();
	}
}

customElements.define("top-level-chat-message", TopLevelChatMessage);

const root = new TopLevelChatMessage();
document.getElementById("chat").appendChild(root);
ChatMessage.messages.push(root);

export default ChatMessage;
export { messageForm };
