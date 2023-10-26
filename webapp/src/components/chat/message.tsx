import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import React, { useState, useEffect } from 'react';
import { ClipboardDocumentIcon } from '@heroicons/react/20/solid';
import dynamic from 'next/dynamic';
// @ts-ignore
const Markdown = dynamic(() => import('react-markdown'), {
	loading: () => <p className='markdown-content'>Loading...</p>,
	ssr: false,
});
import { toast } from 'react-toastify';

export function CopyToClipboardButton({ dataToCopy }) {

	const handleCopyClick = async () => {
	 	try {
			await navigator.clipboard.writeText(dataToCopy);
			toast.success('Copied to clipboard');
		} catch { /* ignored for now */ }
	};

	return (
		<button 
			onClick={handleCopyClick} 
			className='px-1 hover:bg-blue-600 rounded-md shadow focus:outline-none focus:ring-2 focus:ring-blue-300'
			aria-label='Copy to clipboard'
		>
			<ClipboardDocumentIcon className='h-4 w-4 text-white' />
		</button>
	);
}

function CollapsingCodeBody({ messageLanguage, messageContent, style }) {
	const isLongMessage = messageContent
		&& typeof messageContent.split === 'function'
		&& messageContent.split(/\r?\n/).length > 10;
	const [ collapsed, setCollapsed ] = useState(isLongMessage);
	return <>
		<span className='rounded-t overflow-hidden h-8 bg-gray-700 p-2 text-white w-full block text-xs ps-2 flex justify-between'>
			{messageLanguage}
			<CopyToClipboardButton dataToCopy={messageContent} />
		</span>
		<div className='overlay-container'>
			<SyntaxHighlighter
				wrapLongLines
				className={collapsed ? 'overlay-gradient' : null}
				language={messageLanguage}
				style={style}
				showLineNumbers={true}
				customStyle={{ margin: 0, maxHeight: collapsed ? '10em' : 'unset' }}
			>
				{messageContent}
			</SyntaxHighlighter>
			<button
				className='overlay-button btn bg-indigo-600 rounded-md text-white'
				onClick={() => {
					setCollapsed(oldCollapsed => !oldCollapsed);
				}}
			>
				{collapsed ? 'Expand' : 'Collapse'}
			</button>
		</div>
	</>;
}

function MessageBody({ message, messageType, messageLanguage, style }) {
	const messageContent = messageLanguage === 'json'
		? JSON.stringify(message, null, '\t')
		: message.toString();
	switch(messageType) {
		case 'code':
			return <CollapsingCodeBody
				messageContent={messageContent}
				messageLanguage={messageLanguage}
				style={style}
			/>;
		case 'text':
		default:
			return <Markdown
				className={'markdown-content'}
				components={{
					code(props) {
						const { children, className, node, ...rest } = props;
						const match = /language-(\w+)/.exec(className || '');
						return match ? (<CollapsingCodeBody
							messageContent={String(children).replace(/\n$/, '')}
							messageLanguage={match[1]}
							style={style}
						/>) : (<code {...rest} className={className}>
							{children}
						</code>);
					}
				}}
			>
				{message}
			</Markdown>;
	}
}

export function Message({
	prevMessage,
	message,
	messageType,
	messageLanguage,
	isFeedback,
	ts,
	authorName,
	authorImage,
	incoming
}: {
		prevMessage?: any,
		message?: any,
		messageType?: string,
		messageLanguage?: string,
		isFeedback?: boolean,
		ts?: number,
		authorName?: string,
		authorImage?: string,
		incoming?: boolean
	}) {

	const [ style, setStyle ] = useState(null);
	useEffect(() => {
		if (!style) {
			import('react-syntax-highlighter/dist/esm/styles/prism/material-dark')
				.then(mod => setStyle(mod.default));
		}
	});
	
	if (!style) { return null; }

	const sameAuthorAsPrevious = prevMessage && prevMessage.authorName === authorName;

	const profilePicture = <div className={`min-w-max w-9 h-9 rounded-full flex items-center justify-center ${incoming ? 'ms-2' : 'me-2'} select-none`}>
		<span className={`w-8 h-8 rounded-full text-center pt-1 font-bold ring-gray-300 ${!sameAuthorAsPrevious && 'ring-1'}`}>
			{!sameAuthorAsPrevious && authorName.charAt(0).toUpperCase()}
		</span>
	</div>;

	const authorNameSection = !sameAuthorAsPrevious && <div className={`grid grid-cols-1 xl:grid-cols-5 ${prevMessage && !sameAuthorAsPrevious ? 'border-t mt-4' : ''} ${incoming ? 'bg-white' : 'bg-gray-50'}`}>
		<div className='invisible xl:visible col-span-1'></div>
		<small className={`flex px-2 pt-4 col-span-1 xl:col-span-3 ${incoming ? 'justify-end' : ''}`}>
			<strong className='capitalize pe-1'>{authorName}</strong>
		</small>
		<div className='invisible xl:visible col-span-1'></div>
	</div>;

	return <>
		{authorNameSection}
		<div className={`grid grid-cols-1 xl:grid-cols-5 ${incoming ? 'bg-white' : 'bg-gray-50'}`}>
			<div className='invisible xl:visible col-span-1'></div>
			<div className={`flex ${incoming ? 'pe-2 justify-end' : 'ps-2 justify-start'} px-4 pt-1 col-span-1 xl:col-span-3`}>
				{!incoming && profilePicture}
				<div className={`flex max-w-96 ${incoming ? 'bg-indigo-500' : 'bg-white'} rounded-lg ${messageType !== 'code' ? 'px-3 py-2' : 'p-2'} overflow-x-auto`}>
					<p className={`${incoming ? 'text-white' : ''} w-full`}>
						<MessageBody message={message} messageType={messageType} messageLanguage={messageLanguage} style={style} />
						<small className={`flex justify-end pt-1 ${incoming ? 'text-indigo-300' : 'text-gray-500'}`}>
							<time dateTime={new Date(ts).toISOString()}>{new Date(ts).toLocaleString()}</time>
						</small>
					</p>
				</div>
				{incoming && profilePicture}
			</div>
			<div className='invisible xl:visible col-span-1'></div>
		</div>
	</>;

}
