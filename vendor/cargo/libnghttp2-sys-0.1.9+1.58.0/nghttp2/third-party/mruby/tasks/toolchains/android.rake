require "json"

class MRuby::Toolchain::Android

  DEFAULT_ARCH = 'armeabi' # TODO : Revise if arch should have a default

  DEFAULT_TOOLCHAIN = :clang

  DEFAULT_NDK_HOMES = %w{
    /usr/local/opt/android-sdk/ndk-bundle
    /usr/local/opt/android-ndk
    ~/Android/Sdk/ndk-bundle
    %LOCALAPPDATA%/Android/android-sdk/ndk-bundle
    %LOCALAPPDATA%/Android/android-ndk
    %LOCALAPPDATA%/Android/Sdk/ndk/*
    ~/Library/Android/sdk/ndk-bundle
    ~/Library/Android/ndk
  }

  TOOLCHAINS = [:clang, :gcc]

  ARCHITECTURES = %w{
    armeabi armeabi-v7a arm64-v8a
    x86 x86_64
    mips mips64
  }

  class AndroidNDKHomeNotFound < StandardError
    def message
        <<-EOM
Couldn't find Android NDK Home.
Set ANDROID_NDK_HOME environment variable or set :ndk_home parameter
        EOM
    end
  end

  attr_reader :params

  def initialize(params)
    @params = params
  end

  def bin_gcc(command)
    command = command.to_s

    command = case arch
      when /armeabi/    then 'arm-linux-androideabi-'
      when /arm64-v8a/  then 'aarch64-linux-android-'
      when /x86_64/     then 'x86_64-linux-android-'
      when /x86/        then 'i686-linux-android-'
      when /mips64/     then 'mips64el-linux-android-'
      when /mips/       then 'mipsel-linux-android-'
      end + command

    gcc_toolchain_path.join('bin', command).to_s
  end

  def bin(command)
    command = command.to_s
    toolchain_path.join('bin', command).to_s
  end

  def home_path
    @home_path ||= Pathname.new(
      params[:ndk_home] ||
      ENV['ANDROID_NDK_HOME'] ||
      DEFAULT_NDK_HOMES.find { |path|
        path.gsub! '%LOCALAPPDATA%', ENV['LOCALAPPDATA'] || '%LOCALAPPDATA%'
        path.gsub! '\\', '/'
        path.gsub! '~', Dir.home || '~'
        path.gsub!('*') do
          next nil unless path[-1] == "*"
          dirs = Dir.glob(path).collect do |d|
            m = d.match(/(\d+)\.(\d+)\.(\d+)$/)
            m ? [m[1], m[2], m[3]].collect(&:to_i) : nil
          end
          dirs.compact!
          dirs.sort! do |before, after|
            f = 0
            if (f = (after.first <=> before.first)) != 0
              next f
            elsif (f = (after[1] <=> before[1])) != 0
              next f
            else
              next after.last <=> before.last
            end
          end
          dirs.empty? ? nil.to_s : dirs.first.join(".")
        end
        File.directory?(path)
      } || raise(AndroidNDKHomeNotFound)
    )
  end

  def toolchain
    @toolchain ||= params.fetch(:toolchain){ DEFAULT_TOOLCHAIN }
  end

  def toolchain_path
    @toolchain_path ||= case toolchain
      when :gcc
        gcc_toolchain_path
      when :clang
        home_path.join('toolchains', 'llvm' , 'prebuilt', host_platform)
      end
  end

  def gcc_toolchain_path
    if @gcc_toolchain_path === nil then
      prefix = case arch
        when /armeabi/    then 'arm-linux-androideabi-'
        when /arm64-v8a/  then 'aarch64-linux-android-'
        when /x86_64/     then 'x86_64-'
        when /x86/        then 'x86-'
        when /mips64/     then 'mips64el-linux-android-'
        when /mips/       then 'mipsel-linux-android-'
        end

      test = case arch
        when /armeabi/    then 'arm-linux-androideabi-*'
        when /arm64-v8a/  then 'aarch64-linux-android-*'
        when /x86_64/     then 'x86_64-*'
        when /x86/        then 'x86-*'
        when /mips64/     then 'mips64el-linux-android-*'
        when /mips/       then 'mipsel-linux-android-*'
        end

      gcc_toolchain_version = Dir[home_path.join('toolchains', test)].map{|t| t.match(/-(\d+\.\d+)$/); $1.to_f }.max
      @gcc_toolchain_path = home_path.join('toolchains', prefix + gcc_toolchain_version.to_s, 'prebuilt', host_platform)
    end
    @gcc_toolchain_path
  end

  def host_platform
    @host_platform ||= case RUBY_PLATFORM
      when /cygwin|mswin|mingw|bccwin|wince|emx/i
        path = home_path.join('toolchains', 'llvm' , 'prebuilt', 'windows*')
        Dir.glob(path.to_s){ |item|
          next if File.file?(item)
          path = Pathname.new(item)
          break
        }
        path.basename
      when /x86_64-darwin/i
        'darwin-x86_64'
      when /darwin/i
        'darwin-x86'
      when /x86_64-linux/i
        'linux-x86_64'
      when /linux/i
        'linux-x86'
      else
        raise NotImplementedError, "Unknown host platform (#{RUBY_PLATFORM})"
      end
  end

  def arch
    @arch ||= (params[:arch] || ENV['ANDROID_ARCH'] || DEFAULT_ARCH).to_s
  end

  def armeabi_v7a_mfpu
    @armeabi_v7a_mfpu ||= (params[:mfpu] || 'vfpv3-d16').to_s
  end

  def armeabi_v7a_mfloat_abi
    @armeabi_v7a_mfloat_abi ||= (params[:mfloat_abi] || 'softfp').to_s
  end

  def sdk_version
    @sdk_version ||= params[:sdk_version]
    if !@sdk_version then
      # Higher SDK version will be used.
      json = nil
      File.open(home_path + "meta/platforms.json") do |f|
        json = JSON.load(f)
      end
      @sdk_version = json["max"]
    end
    @sdk_version
  end

  def no_warn_mismatch
    if %W(soft softfp).include? armeabi_v7a_mfloat_abi
      ''
    else
      ',--no-warn-mismatch'
    end
  end

  def cc
    case toolchain
    when :gcc then bin_gcc('gcc')
    when :clang then bin('clang')
    end
  end

  def ar
    case toolchain
    when :gcc   then bin_gcc('ar')
    when :clang then bin_gcc('ar')
    end
  end

  def ctarget
    flags = []

    v = sdk_version
    case toolchain
    when :gcc
      case arch
      when /armeabi-v7a/  then flags += %W(-march=armv7-a)
      when /armeabi/      then flags += %W(-march=armv5te)
      when /arm64-v8a/    then flags += %W(-march=armv8-a)
      when /x86_64/       then flags += %W(-march=x86-64)
      when /x86/          then flags += %W(-march=i686)
      when /mips64/       then flags += %W(-march=mips64r6)
      when /mips/         then flags += %W(-march=mips32)
      end
    when :clang
      case arch
      when /armeabi-v7a/  then flags += %W(-target armv7-none-linux-androideabi#{v})
      when /armeabi/      then flags += %W(-target armv5te-none-linux-androideabi#{v})
      when /arm64-v8a/    then flags += %W(-target aarch64-none-linux-android#{v})
      when /x86_64/       then flags += %W(-target x86_64-none-linux-android#{v})
      when /x86/          then flags += %W(-target i686-none-linux-android#{v})
      when /mips64/       then flags += %W(-target mips64el-none-linux-android#{v})
      when /mips/         then flags += %W(-target mipsel-none-linux-android#{v})
      end
    end

    case arch
    when /armeabi-v7a/  then flags += %W(-mfpu=#{armeabi_v7a_mfpu} -mfloat-abi=#{armeabi_v7a_mfloat_abi})
    when /armeabi/      then flags += %W(-mtune=xscale -msoft-float)
    when /arm64-v8a/    then flags += %W()
    when /x86_64/       then flags += %W()
    when /x86/          then flags += %W()
    when /mips64/       then flags += %W(-fmessage-length=0)
    when /mips/         then flags += %W(-fmessage-length=0)
    end

    flags
  end

  def cflags
    flags = []

    case RUBY_PLATFORM
    when /mswin|mingw|win32/
      # Build for Android don't need window flag
      flags += %W(-U_WIN32 -U_WIN64)
    end

    flags += %W(-MMD -MP -D__android__ -DANDROID)
    flags += ctarget
    case toolchain
    when :gcc
    when :clang
      flags += %W(-gcc-toolchain "#{gcc_toolchain_path}" -Wno-invalid-command-line-argument -Wno-unused-command-line-argument)
    end
    flags += %W(-fpic -ffunction-sections -funwind-tables -fstack-protector-strong -no-canonical-prefixes)

    flags
  end

  def ldflags
    flags = []

    flags
  end

  def ldflags_before_libraries
    flags = []

    v = sdk_version
    case toolchain
    when :gcc
      case arch
      when /armeabi-v7a/  then flags += %W(-Wl#{no_warn_mismatch})
      end
    when :clang
      flags += %W(-gcc-toolchain "#{gcc_toolchain_path.to_s}")
      case arch
      when /armeabi-v7a/  then flags += %W(-target armv7-none-linux-androideabi#{v} -Wl,--fix-cortex-a8#{no_warn_mismatch})
      when /armeabi/      then flags += %W(-target armv5te-none-linux-androideabi#{v})
      when /arm64-v8a/    then flags += %W(-target aarch64-none-linux-android#{v})
      when /x86_64/       then flags += %W(-target x86_64-none-linux-android#{v})
      when /x86/          then flags += %W(-target i686-none-linux-android#{v})
      when /mips64/       then flags += %W(-target mips64el-none-linux-android#{v})
      when /mips/         then flags += %W(-target mipsel-none-linux-android#{v})
      end
    end
    flags += %W(-no-canonical-prefixes)

    flags
  end
end

MRuby::Toolchain.new(:android) do |conf, params|
  android = MRuby::Toolchain::Android.new(params)

  toolchain android.toolchain

  [conf.cc, conf.cxx, conf.objc, conf.asm].each do |cc|
    cc.command = android.cc
    cc.flags = android.cflags
  end

  conf.archiver.command = android.ar
  conf.linker.command = android.cc
  conf.linker.flags = android.ldflags
  conf.linker.flags_before_libraries = android.ldflags_before_libraries
end
