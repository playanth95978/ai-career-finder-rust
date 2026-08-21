import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { FormsModule } from '@angular/forms';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { NgbActiveModal } from '@ng-bootstrap/ng-bootstrap/modal';

import { ITEM_DELETED_EVENT } from 'app/config/navigation.constants';
import { AlertError } from 'app/shared/alert/alert-error';
import { TranslateDirective } from 'app/shared/language';
import { IOfferTailoredResume } from '../offer-tailored-resume.model';
import { OfferTailoredResumeService } from '../service/offer-tailored-resume.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './offer-tailored-resume-delete-dialog.html',
  imports: [TranslateDirective, FormsModule, FontAwesomeModule, AlertError],
})
export class OfferTailoredResumeDeleteDialog {
  offerTailoredResume?: IOfferTailoredResume;

  protected readonly offerTailoredResumeService = inject(OfferTailoredResumeService);
  protected readonly activeModal = inject(NgbActiveModal);

  cancel(): void {
    this.activeModal.dismiss();
  }

  confirmDelete(id: number): void {
    this.offerTailoredResumeService.delete(id).subscribe(() => {
      this.activeModal.close(ITEM_DELETED_EVENT);
    });
  }
}
