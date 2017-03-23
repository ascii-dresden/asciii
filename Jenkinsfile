def environment(args) {
	sh "ci/environment " + args
}

stage("Build") {
	parallel (
		"linux": {
			node("linux && docker") {
				checkout([$class: 'GitSCM', 
					branches: [[name: "${env.BRANCH_NAME}"]], 
					doGenerateSubmoduleConfigurations: false, 
					extensions: [[$class: 'CleanCheckout']], 
					submoduleCfg: [], 
					userRemoteConfigs: [[url: 'https://github.com/ascii-dresden/asciii.git']]
				])
				environment("cargo build --release")
				environment("strip -s target/release/asciii")
				sh("test -f output/asciii-linux && exit 0; mkdir output; cp target/release/asciii output/asciii-linux")
				archiveArtifacts artifacts: "output/*"
			}
		},
		"osx": {
			node("osx && rust") {
				checkout([$class: 'GitSCM', 
					branches: [[name: "${env.BRANCH_NAME}"]], 
					doGenerateSubmoduleConfigurations: false, 
					extensions: [[$class: 'CleanCheckout']], 
					submoduleCfg: [], 
					userRemoteConfigs: [[url: 'https://github.com/ascii-dresden/asciii.git']]
				])
				environment("cargo build --release")
				environment("strip target/release/asciii")
				sh("test -f output/asciii-macos && exit 0; mkdir output; cp target/release/asciii output/asciii-macos")
				archiveArtifacts artifacts: "output/*"
			}
		},
		"win64": {
			node("win64 && rust") {
				checkout([$class: 'GitSCM', 
					branches: [[name: "${env.BRANCH_NAME}"]], 
					doGenerateSubmoduleConfigurations: false, 
					extensions: [[$class: 'CleanCheckout']], 
					submoduleCfg: [], 
					userRemoteConfigs: [[url: 'https://github.com/ascii-dresden/asciii.git']]
				])
				// TODO: fix line-endings
				sh("sed -i 's/\\r\$//' ci/environment")
				// builds on stable fail because of reasons, so use nighly for now
				environment("rustup run nightly-x86_64-pc-windows-msvc cargo build --release")
				sh("test -f output/asciii-windows.exe && exit 0; mkdir output; cp target/release/asciii.exe output/asciii-windows.exe")
				archiveArtifacts artifacts: "output/*"
			}	
		}
	)	
}
