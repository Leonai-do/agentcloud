import { CubeIcon } from '@heroicons/react/24/outline';
import animationData from 'animations/SignInAnimationTransparent.json';
import dynamic from 'next/dynamic';
import { useEffect, useRef } from 'react';
import { ReactNode } from 'react-markdown';

const Lottie = dynamic(() => import('lottie-react'), {
	ssr: false
});

const LeftFrame = ({
	children,
	showAnimation = false
}: {
	children: ReactNode;
	showAnimation?: boolean;
}) => {
	const lottieRef = useRef<any>(null);

	const handlePauseAnimation = () => {
		if (lottieRef.current) {
			lottieRef.current.pause();
		}
	};

	useEffect(() => {
		if (!showAnimation) {
			handlePauseAnimation();
		}
	}, [showAnimation]);
	console.log(lottieRef);

	return (
		<div className='w-full gradient-animation max-w-xl text-white pt-12 px-12 flex flex-col relative'>
			<Lottie
				lottieRef={lottieRef}
				animationData={animationData}
				loop={true}
				className='absolute top-[-200px] left-0 w-full h-full opacity-30'
			/>
			<div className='flex items-center gap-x-2'>
				<CubeIcon className='h-5 w-5 stroke-2' />
				<h1 className='text-2xl font-bold'>Agent Cloud</h1>
			</div>

			{children}
		</div>
	);
};

export default LeftFrame;