// Board filter component for filtering red cards
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { AlertCircle } from 'lucide-react';

interface BoardFilterProps {
    onlyRedCards: boolean;
    onOnlyRedCardsChange: (value: boolean) => void;
    redCardCount?: number;
}

export function BoardFilter({
    onlyRedCards,
    onOnlyRedCardsChange,
    redCardCount,
}: BoardFilterProps) {
    return (
        <div className="flex items-center gap-2">
            <Switch
                id="red-card-filter"
                checked={onlyRedCards}
                onCheckedChange={onOnlyRedCardsChange}
            />
            <Label
                htmlFor="red-card-filter"
                className="flex items-center gap-1.5 cursor-pointer select-none text-sm"
            >
                <AlertCircle className="h-4 w-4 text-red-500" />
                只看红卡
            </Label>
            {typeof redCardCount === 'number' && redCardCount > 0 && (
                <Badge variant="destructive" className="ml-1">
                    {redCardCount}
                </Badge>
            )}
        </div>
    );
}
