import { BotIcon, ChevronDownIcon } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"

export function AiChatPopover() {
  return (
    <div className="flex items-center space-x-1">
      <Button variant="outline">
        <BotIcon className="mr-2" /> KYS Asistan
      </Button>
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="icon" aria-label="Open Popover">
            <ChevronDownIcon />
          </Button>
        </PopoverTrigger>
        <PopoverContent align="end" className="rounded-xl text-sm w-80">
          <div className="flex flex-col space-y-1 mb-4">
            <h4 className="font-semibold leading-none">Yapay Zeka ile İncele</h4>
            <p className="text-sm text-muted-foreground">
              Proje hakkında yapay zekaya dilediğinizi sorabilirsiniz.
            </p>
          </div>
          <div className="grid gap-2">
            <label htmlFor="task" className="sr-only">Görev Açıklaması</label>
            <textarea
              id="task"
              placeholder="Örn: Bu projenin en büyük eksikliği nedir?"
              className="resize-none mt-2 flex min-h-[80px] w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
            />
            <p className="text-sm text-muted-foreground mt-2">
              KYS Asistan, analiz raporunu sizin için yorumlayacaktır.
            </p>
          </div>
        </PopoverContent>
      </Popover>
    </div>
  )
}
